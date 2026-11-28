//! A rács adatmodellje: a Slint ListView csak a látható sorokat kéri el, és a hiányzó
//! bélyegképek ekkor kerülnek a sürgős munkasorba.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

use slint::{
    Image, Model, ModelNotify, ModelRc, ModelTracker, Rgb8Pixel, SharedPixelBuffer, SharedString,
    VecModel,
};

use crate::elrendezes::{self, SorTipus, Tartomany};
use crate::feladat::Sorban;
use crate::munkasok::Munka;
use crate::tipusok::{Foto, Mappa};
use crate::{Cella, Sor};
use pr_thumbs::belyegkep::Pixelek;

/// Ennyi bélyegkép marad a memóriában; a többi a lemezes gyorstárból jön újra.
const MEMORIABAN: usize = 600;

pub fn kep_slintbe(p: &Pixelek) -> Image {
    Image::from_rgb8(SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        &p.rgb, p.szel, p.mag,
    ))
}

/// A hibás kép helye: így nem kérjük újra meg újra.
fn hibas_kep() -> Image {
    kep_slintbe(&Pixelek {
        szel: 4,
        mag: 4,
        rgb: vec![160; 4 * 4 * 3],
    })
}

pub struct MappaAdat {
    ut: std::path::PathBuf,
    pub nev: String,
    pub tartomany: Tartomany,
    teljes_db: usize,
}

pub struct Allapot {
    pub fotok: Vec<Foto>,
    pub mappak: Vec<MappaAdat>,
    pub sorok: Vec<SorTipus>,
    pub oszlop: usize,
    pub nezett: Option<usize>,
    kepek: HashMap<usize, Image>,
    kep_sorrend: VecDeque<usize>,
    kert: HashSet<usize>,
    munkak: Arc<Sorban<Munka>>,
}

impl Allapot {
    pub fn uj(oszlop: usize, munkak: Arc<Sorban<Munka>>) -> Self {
        Self {
            fotok: Vec::new(),
            mappak: Vec::new(),
            sorok: Vec::new(),
            oszlop,
            nezett: None,
            kepek: HashMap::new(),
            kep_sorrend: VecDeque::new(),
            kert: HashSet::new(),
            munkak,
        }
    }

    fn tartomanyok(&self) -> Vec<Tartomany> {
        self.mappak.iter().map(|m| m.tartomany).collect()
    }

    /// Új mappa a lista végére (a meglévő sorok nem mozdulnak); visszaadja az új képeket.
    pub fn hozzaad(&mut self, mappa: Mappa, teljes_db: usize) -> (bool, Vec<(usize, Foto)>) {
        let elso = self.fotok.len();
        let folytatas = self
            .mappak
            .last()
            .is_some_and(|utolso| utolso.ut == mappa.ut);
        if folytatas {
            if let Some(utolso) = self.mappak.last_mut() {
                utolso.tartomany.db += mappa.fotok.len();
            }
        } else {
            self.mappak.push(MappaAdat {
                ut: mappa.ut,
                nev: mappa.nev,
                tartomany: Tartomany {
                    elso,
                    db: mappa.fotok.len(),
                },
                teljes_db,
            });
        }
        self.fotok.extend(mappa.fotok.iter().cloned());
        self.sorok = elrendezes::sorok(&self.tartomanyok(), self.oszlop);
        (!folytatas, (elso..).zip(mappa.fotok).collect())
    }

    /// Új oszlopszám; visszaadja, melyik sor legyen felül.
    pub fn atrendez(&mut self, oszlop: usize, felso: usize) -> usize {
        let uj = elrendezes::sorok(&self.tartomanyok(), oszlop);
        let cel = elrendezes::horgony(&self.sorok, &uj, felso);
        self.sorok = uj;
        self.oszlop = oszlop;
        cel
    }

    /// A megérkezett bélyegkép eltárolása; visszaadja az érintett sort.
    pub fn kep_erkezett(&mut self, id: usize, kep: Option<Pixelek>) -> Option<usize> {
        self.kert.remove(&id);
        let kep = kep.map(|p| kep_slintbe(&p)).unwrap_or_else(hibas_kep);
        if self.kepek.insert(id, kep).is_none() {
            self.kep_sorrend.push_back(id);
        }
        while self.kep_sorrend.len() > MEMORIABAN {
            if let Some(regi) = self.kep_sorrend.pop_front() {
                self.kepek.remove(&regi);
            }
        }
        elrendezes::foto_sora(&self.sorok, id)
    }

    pub fn megnyit(&mut self, id: usize) -> Option<(Foto, Option<Image>)> {
        let foto = self.fotok.get(id)?.clone();
        self.nezett = Some(id);
        Some((foto, self.kepek.get(&id).cloned()))
    }

    fn cella(&mut self, id: usize) -> Cella {
        if let Some(kep) = self.kepek.get(&id) {
            return Cella {
                kep: kep.clone(),
                van: true,
                id: id as i32,
            };
        }
        if self.kert.insert(id) {
            self.munkak.surgos(Munka {
                id,
                foto: self.fotok[id].clone(),
                mutat: true,
            });
        }
        Cella {
            kep: Image::default(),
            van: false,
            id: id as i32,
        }
    }

    fn sor_adat(&mut self, i: usize) -> Option<Sor> {
        Some(match self.sorok.get(i)?.clone() {
            SorTipus::Fejlec { mappa } => {
                let m = &self.mappak[mappa];
                Sor {
                    fejlec: true,
                    cim: format!("{} ({})", m.nev, m.teljes_db).into(),
                    cellak: ModelRc::default(),
                }
            }
            SorTipus::Kepek { elso, db } => {
                let cellak: Vec<Cella> = (elso..elso + db).map(|id| self.cella(id)).collect();
                Sor {
                    fejlec: false,
                    cim: SharedString::default(),
                    cellak: ModelRc::new(VecModel::from(cellak)),
                }
            }
        })
    }
}

pub struct RacsModell {
    allapot: Rc<RefCell<Allapot>>,
    jelzo: ModelNotify,
}

impl RacsModell {
    pub fn uj(allapot: Rc<RefCell<Allapot>>) -> Self {
        Self {
            allapot,
            jelzo: ModelNotify::default(),
        }
    }

    // A hívó ezek előtt engedje el az állapot kölcsönzését: a jelzés után sorokat kérnek el.
    pub fn sor_valtozott(&self, sor: usize) {
        self.jelzo.row_changed(sor);
    }

    pub fn sorok_hozzaadva(&self, elso: usize, db: usize) {
        self.jelzo.row_added(elso, db);
    }

    pub fn ujraepit(&self) {
        self.jelzo.reset();
    }
}

impl Model for RacsModell {
    type Data = Sor;

    fn row_count(&self) -> usize {
        self.allapot.borrow().sorok.len()
    }

    fn row_data(&self, sor: usize) -> Option<Sor> {
        self.allapot.borrow_mut().sor_adat(sor)
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.jelzo
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tesztek {
    use super::*;
    use crate::feladat::Sorban;

    #[test]
    fn egy_mappa_utemei_egy_fejlec_alatt_maradnak() {
        let mut allapot = Allapot::uj(4, Arc::new(Sorban::uj()));
        let (uj_mappa, elso) = allapot.hozzaad(
            Mappa {
                ut: "album".into(),
                nev: "album".to_string(),
                fotok: vec![Foto {
                    ut: "album/1.jpg".into(),
                    meret: 1,
                    mtime_ns: 0,
                }],
            },
            2,
        );
        let (folytatas, masodik) = allapot.hozzaad(
            Mappa {
                ut: "album".into(),
                nev: "album".to_string(),
                fotok: vec![Foto {
                    ut: "album/2.jpg".into(),
                    meret: 2,
                    mtime_ns: 0,
                }],
            },
            2,
        );

        assert!(uj_mappa);
        assert_eq!(elso.len(), 1);
        assert!(!folytatas);
        assert_eq!(masodik[0].0, 1);
        assert_eq!(allapot.mappak.len(), 1);
        assert_eq!(allapot.mappak[0].teljes_db, 2);
        assert_eq!(allapot.fotok.len(), 2);
    }
}
