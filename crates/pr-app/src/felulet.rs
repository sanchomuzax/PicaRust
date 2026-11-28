//! A Slint-ablak összekötése a könyvtárral és a háttérszálakkal.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use slint::{ComponentHandle, Image, ModelRc, Timer, TimerMode, VecModel};

use crate::beallitas::Beallitas;
use crate::feladat::Sorban;
use crate::konyvtar::{self, Mappa};
use crate::munkasok::{self, ArcSzamlalo, Eredmeny, Munka};
use crate::racsmodell::{kep_slintbe, Allapot, RacsModell};
use crate::{arcnaplo, elrendezes, naplo, AppWindow, MappaElem};

/// Az indításkori 1000-230 képpontos rácshoz; az első ütem a valódi szélességre igazít.
const KEZDO_OSZLOP: usize = 4;

/// Amit az ablak időzítője tart a kezében.
struct Kezelo {
    allapot: Rc<RefCell<Allapot>>,
    modell: Rc<RacsModell>,
    oldalsav: Rc<VecModel<MappaElem>>,
    be: Receiver<Eredmeny>,
    munkak: Arc<Sorban<Munka>>,
    arc_ki: Sender<konyvtar::Foto>,
    arc_szamlalo: Arc<ArcSzamlalo>,
    arc_kesz_kulcsok: HashSet<String>,
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
                if !self.arc_kesz_kulcsok.contains(&arcnaplo::kulcs(foto)) {
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
                Eredmeny::UjMappak(mappak) => uj_mappak.extend(mappak),
            }
        }
        for sor in erintett {
            self.modell.sor_valtozott(sor);
        }
        if !uj_mappak.is_empty() {
            self.felvesz(uj_mappak);
        }
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

pub fn futtat(beallitas: &Beallitas) -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    let mappak = konyvtar::fa_beolvas(&beallitas.gyoker);
    naplo(&format!(
        "beolvasas: {} mappa, {} kep",
        mappak.len(),
        mappak.iter().map(|m| m.fotok.len()).sum::<usize>()
    ));

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

    let ismert = ismert_almappak(&beallitas.gyoker, &mappak);
    let mut kezelo = Kezelo {
        allapot: Rc::clone(&allapot),
        modell,
        oldalsav,
        be: eredmeny_be,
        munkak,
        arc_ki,
        arc_szamlalo,
        arc_kesz_kulcsok: arcnaplo::beolvas(&arcnaplo_ut),
        utolso_allapotsor: Instant::now(),
    };
    kezelo.felvesz(mappak);
    kezelo.allapotsor(&app);
    munkasok::indit_mappafigyelo(beallitas.gyoker.clone(), ismert, eredmeny_ki.clone());

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
