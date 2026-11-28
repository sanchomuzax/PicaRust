//! A Slint-ablak összekötése a könyvtárral és a háttérszálakkal.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, Image, ModelRc, Timer, TimerMode, VecModel};

use crate::beallitas::Beallitas;
use crate::feladat::Sorban;
use crate::munkasok::{self, ArcSzamlalo, Eredmeny, Foto, InduloAdat, Mappa, Munka};
use crate::racsmodell::{kep_slintbe, Allapot, RacsModell};
use crate::{elrendezes, naplo, AppWindow, MappaElem};
use pr_faces::arcnaplo;

/// Az indításkori 1000-230 képpontos rácshoz; az első ütem a valódi szélességre igazít.
const KEZDO_OSZLOP: usize = 4;
const MAX_MAPPANKENTI_UTEM: usize = 32;

#[derive(Default)]
struct InduloAllapot {
    kesz: bool,
    hiba: Option<String>,
    arc_kesz_kulcsok: HashSet<String>,
}

struct InduloMuvelet {
    mappak: Vec<Mappa>,
    ismert_almappak: HashSet<PathBuf>,
    figyelot_indit: bool,
}

impl InduloAllapot {
    /// A háttérből kapott induló üzenetből kiszámítja a felület teendőit.
    fn kezeld(&mut self, gyoker: &Path, eredmeny: Result<InduloAdat, String>) -> InduloMuvelet {
        let (mappak, arc_kesz_kulcsok, hiba) = match eredmeny {
            Ok(adat) => (adat.mappak, adat.arc_kesz_kulcsok, None),
            Err(hiba) => (Vec::new(), HashSet::new(), Some(hiba)),
        };
        self.kesz = true;
        self.hiba = hiba;
        self.arc_kesz_kulcsok = arc_kesz_kulcsok;
        let ismert_almappak = ismert_almappak(gyoker, &mappak);
        InduloMuvelet {
            mappak,
            ismert_almappak,
            figyelot_indit: true,
        }
    }

    fn allapotszoveg(&self) -> Option<String> {
        self.hiba
            .as_ref()
            .map(|hiba| format!("A képtár betöltése nem sikerült: {hiba}"))
    }
}

/// Amit az ablak időzítője tart a kezében.
struct Kezelo {
    allapot: Rc<RefCell<Allapot>>,
    modell: Rc<RacsModell>,
    oldalsav: Rc<VecModel<MappaElem>>,
    be: Receiver<Eredmeny>,
    eredmeny_ki: Sender<Eredmeny>,
    munkak: Arc<Sorban<Munka>>,
    gyoker: PathBuf,
    indulo: InduloAllapot,
    mappa_sor: VecDeque<Mappa>,
    arc_ki: Sender<Foto>,
    arc_szamlalo: Arc<ArcSzamlalo>,
    utolso_allapotsor: Instant,
}

impl Kezelo {
    /// Felveszi a mappákat, és sorba teszi a bélyegkép- és arckereső munkát.
    fn felvesz(&mut self, mappak: Vec<Mappa>) {
        let regi_sorok = self.allapot.borrow().sorok.len();
        for mappa in mappak {
            self.oldalsav.push(MappaElem {
                nev: mappa.nev.clone().into(),
                db: mappa.fotok.len() as i32,
            });
            let ujak = self.allapot.borrow_mut().hozzaad(mappa);
            for (_, foto) in &ujak {
                if !self.indulo.arc_kesz_kulcsok.contains(&arcnaplo::kulcs(
                    &foto.ut,
                    foto.meret,
                    foto.mtime_ns,
                )) {
                    self.arc_szamlalo.osszes.fetch_add(1, Ordering::SeqCst);
                    // az arckereső szál a program végéig él
                    if self.arc_ki.send(foto.clone()).is_err() {
                        naplo("az arckereso szal leallt");
                    }
                }
            }
            self.munkak.hatter(ujak.into_iter().map(|(id, foto)| Munka {
                id,
                foto,
                mutat: false,
            }));
        }
        let uj_sorok = self.allapot.borrow().sorok.len();
        if uj_sorok > regi_sorok {
            self.modell
                .sorok_hozzaadva(regi_sorok, uj_sorok - regi_sorok);
        }
    }

    fn utem(&mut self, app: &AppWindow) {
        self.eredmenyek(app);
        let ut_mappak = kovetkezo_mappa_utem(&mut self.mappa_sor);
        if !ut_mappak.is_empty() {
            self.felvesz(ut_mappak);
        }
        self.atmeretezes(app);
        if self.utolso_allapotsor.elapsed() >= Duration::from_millis(500) {
            self.utolso_allapotsor = Instant::now();
            self.allapotsor(app);
        }
    }

    fn eredmenyek(&mut self, app: &AppWindow) {
        let mut erintett = BTreeSet::new();
        let mut uj_mappak = Vec::new();
        while let Ok(eredmeny) = self.be.try_recv() {
            match eredmeny {
                Eredmeny::Belyegkep { id, kep } => {
                    erintett.extend(self.allapot.borrow_mut().kep_erkezett(id, kep));
                }
                Eredmeny::Nagy { id, kep } => {
                    let nezett = self.allapot.borrow().nezett;
                    if let (true, Some(kep)) = (nezett == Some(id), kep) {
                        app.set_nezo_kep(kep_slintbe(&kep));
                    }
                }
                Eredmeny::Indulo(eredmeny) => {
                    let teendok = self.indulo.kezeld(&self.gyoker, eredmeny);
                    if teendok.figyelot_indit {
                        munkasok::indit_mappafigyelo(
                            self.gyoker.clone(),
                            teendok.ismert_almappak,
                            self.eredmeny_ki.clone(),
                        );
                    }
                    uj_mappak.extend(teendok.mappak);
                }
                Eredmeny::UjMappak(mappak) => uj_mappak.extend(mappak),
            }
        }
        for sor in erintett {
            self.modell.sor_valtozott(sor);
        }
        self.mappa_sor.extend(uj_mappak);
    }

    /// Más ablakszélességnél új oszlopszám, a felül látott tartalom marad felül.
    fn atmeretezes(&mut self, app: &AppWindow) {
        let szelesseg = app.get_racs_szelesseg();
        let oszlop = elrendezes::oszlopok(szelesseg);
        let eddigi = self.allapot.borrow().oszlop;
        if szelesseg <= 0.0 || oszlop == eddigi {
            return;
        }
        let felso = elrendezes::felso_sor(app.get_gorgetes());
        let (cel, sor_db) = {
            let mut allapot = self.allapot.borrow_mut();
            let cel = allapot.atrendez(oszlop, felso);
            (cel, allapot.sorok.len())
        };
        self.modell.ujraepit();
        app.set_gorgetes(elrendezes::gorgetes_sorhoz(
            cel,
            sor_db,
            app.get_racs_magassag(),
        ));
    }

    fn allapotsor(&self, app: &AppWindow) {
        if let Some(szoveg) = self.indulo.allapotszoveg() {
            app.set_allapot(szoveg.into());
            return;
        }
        if !self.indulo.kesz {
            app.set_allapot("Képtár betöltése...".into());
            return;
        }
        let (kepek, mappak) = {
            let a = self.allapot.borrow();
            (a.fotok.len(), a.mappak.len())
        };
        let szoveg = format!(
            "{kepek} kép, {mappak} mappa   várakozó bélyegkép: {}   arckeresés: {}/{}",
            self.munkak.hossz(),
            self.arc_szamlalo.kesz.load(Ordering::SeqCst),
            self.arc_szamlalo.osszes.load(Ordering::SeqCst),
        );
        // változatlan érték nem rajzoltat újra
        app.set_allapot(szoveg.into());
    }
}

/// A gyökér közvetlen almappái, amelyek alatt a mappák már ismertek.
fn ismert_almappak(gyoker: &Path, mappak: &[Mappa]) -> HashSet<PathBuf> {
    mappak
        .iter()
        .filter_map(|m| m.ut.strip_prefix(gyoker).ok())
        .filter_map(|r| r.components().next())
        .map(|c| gyoker.join(c))
        .collect()
}

fn kovetkezo_mappa_utem(mappak: &mut VecDeque<Mappa>) -> Vec<Mappa> {
    (0..MAX_MAPPANKENTI_UTEM)
        .filter_map(|_| mappak.pop_front())
        .collect()
}

pub fn futtat(beallitas: &Beallitas) -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;

    let munkak = Arc::new(Sorban::uj());
    let (eredmeny_ki, eredmeny_be) = mpsc::channel();
    let magok = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    naplo(&format!("belyegkep-szalak: {magok}"));
    munkasok::indit_belyegkep_szalak(magok, &munkak, &eredmeny_ki, &beallitas.gyorstar);

    let arc_szamlalo = Arc::new(ArcSzamlalo::default());
    let arcnaplo_ut = beallitas.gyorstar.join("arcok.tsv");
    let arc_ki = munkasok::indit_arckereso(
        beallitas.modell.clone(),
        arcnaplo_ut.clone(),
        Arc::clone(&arc_szamlalo),
    );

    let allapot = Rc::new(RefCell::new(Allapot::uj(KEZDO_OSZLOP, Arc::clone(&munkak))));
    let modell = Rc::new(RacsModell::uj(Rc::clone(&allapot)));
    let oldalsav = Rc::new(VecModel::<MappaElem>::default());
    app.set_sorok(ModelRc::from(Rc::clone(&modell)));
    app.set_mappak(ModelRc::from(Rc::clone(&oldalsav)));

    let mut kezelo = Kezelo {
        allapot: Rc::clone(&allapot),
        modell,
        oldalsav,
        be: eredmeny_be,
        eredmeny_ki: eredmeny_ki.clone(),
        munkak,
        gyoker: beallitas.gyoker.clone(),
        indulo: InduloAllapot::default(),
        mappa_sor: VecDeque::new(),
        arc_ki,
        arc_szamlalo,
        utolso_allapotsor: Instant::now(),
    };
    kezelo.allapotsor(&app);
    munkasok::indit_indulo_beolvasas(beallitas.gyoker.clone(), arcnaplo_ut, eredmeny_ki.clone());

    visszahivasok(&app, &allapot, &eredmeny_ki);

    let idozito = Timer::default();
    let gyenge = app.as_weak();
    idozito.start(TimerMode::Repeated, Duration::from_millis(15), move || {
        if let Some(app) = gyenge.upgrade() {
            kezelo.utem(&app);
        }
    });
    naplo("ablak indul");
    app.run()?;
    naplo("ablak bezarva");
    Ok(())
}

fn visszahivasok(app: &AppWindow, allapot: &Rc<RefCell<Allapot>>, ki: &Sender<Eredmeny>) {
    let (gyenge, a) = (app.as_weak(), Rc::clone(allapot));
    app.on_mappa_valasztva(move |mappa| {
        let Some(app) = gyenge.upgrade() else { return };
        let magassag = app.get_racs_magassag();
        let cel = {
            let a = a.borrow();
            elrendezes::mappa_sora(&a.sorok, mappa as usize)
                .map(|sor| elrendezes::gorgetes_sorhoz(sor, a.sorok.len(), magassag))
        };
        if let Some(y) = cel {
            app.set_aktiv_mappa(mappa);
            app.set_gorgetes(y);
        }
    });

    let (gyenge, a, ki) = (app.as_weak(), Rc::clone(allapot), ki.clone());
    app.on_kep_megnyitva(move |id| {
        let Some(app) = gyenge.upgrade() else { return };
        let id = id as usize;
        let Some((foto, elonezet)) = a.borrow_mut().megnyit(id) else {
            return;
        };
        // a bélyegkép azonnal látszik, a nagy kép a dekódolás után váltja le
        app.set_nezo_cim(foto.ut.display().to_string().into());
        app.set_nezo_kep(elonezet.unwrap_or_default());
        app.set_nezo_nyitva(true);
        munkasok::nagy_kep(id, foto, ki.clone());
    });

    let (gyenge, a) = (app.as_weak(), Rc::clone(allapot));
    app.on_vissza(move || {
        let Some(app) = gyenge.upgrade() else { return };
        a.borrow_mut().nezett = None;
        app.set_nezo_nyitva(false);
        app.set_nezo_kep(Image::default());
    });
}

#[cfg(test)]
mod tesztek {
    use super::*;
    use crate::munkasok::{Foto, InduloAdat, Mappa};

    #[test]
    fn felulet_nem_hivja_a_lemezolvasokat_a_felepitesben() {
        let vege = include_str!("felulet.rs");
        let forras = vege
            .split_once("#[cfg(test)]")
            .map(|(forras, _)| forras)
            .unwrap_or(vege);

        assert!(
            !forras.contains("fa_beolvas"),
            "a felulet fajaban ne legyen mappafa-beolvasas"
        );
        assert!(
            !forras.contains("arcnaplo::beolvas"),
            "a felulet fajaban ne legyen arcnaplo-beolvasas"
        );
        assert!(
            forras.contains("munkasok::indit_indulo_beolvasas"),
            "az indulo lemezbeolvasast a munkasok modul inditsa"
        );
    }

    #[test]
    fn indulo_kezeles_arcnaplo_kulccsal_es_figyelovel_slint_nelkul() {
        let gyoker = Path::new("keptar");
        let foto = Foto {
            ut: gyoker.join("album/foto.jpg"),
            meret: 123,
            mtime_ns: 456,
        };
        let mappa = Mappa {
            ut: gyoker.join("album"),
            nev: "album".to_string(),
            fotok: vec![foto.clone()],
        };
        let kulcs = arcnaplo::kulcs(&foto.ut, foto.meret, foto.mtime_ns);
        let mut allapot = InduloAllapot::default();

        let muvelet = allapot.kezeld(
            gyoker,
            Ok(InduloAdat {
                mappak: vec![mappa.clone()],
                arc_kesz_kulcsok: HashSet::from([kulcs.clone()]),
            }),
        );

        assert!(muvelet.figyelot_indit);
        assert_eq!(
            muvelet.ismert_almappak,
            HashSet::from([gyoker.join("album")])
        );
        assert_eq!(muvelet.mappak, vec![mappa]);
        assert!(allapot.arc_kesz_kulcsok.contains(&kulcs));
        assert_eq!(allapot.allapotszoveg(), None);
    }

    #[test]
    fn indulo_hibanal_magyar_allapotot_es_figyelot_ad() {
        let gyoker = Path::new("keptar");
        let mut allapot = InduloAllapot::default();
        let muvelet = allapot.kezeld(gyoker, Err("teszt hiba".to_string()));

        assert!(muvelet.figyelot_indit);
        assert!(muvelet.mappak.is_empty());
        assert_eq!(
            allapot.allapotszoveg().as_deref(),
            Some("A képtár betöltése nem sikerült: teszt hiba")
        );
    }

    #[test]
    fn egy_idozito_utem_legfeljebb_a_nevezett_mennyisegu_mappat_veszi_fel() {
        let mut mappak = (0..MAX_MAPPANKENTI_UTEM + 2)
            .map(|index| Mappa {
                ut: PathBuf::from(format!("album-{index}")),
                nev: format!("album-{index}"),
                fotok: Vec::new(),
            })
            .collect::<std::collections::VecDeque<_>>();

        let elso_utem = kovetkezo_mappa_utem(&mut mappak);

        assert_eq!(elso_utem.len(), MAX_MAPPANKENTI_UTEM);
        assert_eq!(mappak.len(), 2);
    }
}
