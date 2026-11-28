//! Háttérszálak: bélyegképek (magonként egy), egy arckereső és egy mappafigyelő.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::arc::Detektor;
use crate::belyegkep::{self, Pixelek};
use crate::feladat::Sorban;
use crate::konyvtar::{self, Foto, Mappa};
use crate::{arcnaplo, naplo, prioritas};

pub struct Munka {
    pub id: usize,
    pub foto: Foto,
    /// Igaz, ha a képpontok a felületre kellenek; hamis, ha csak a gyorstár készüljön.
    pub mutat: bool,
}

pub enum Eredmeny {
    Belyegkep { id: usize, kep: Option<Pixelek> },
    Nagy { id: usize, kep: Option<Pixelek> },
    UjMappak(Vec<Mappa>),
}

#[derive(Default)]
pub struct ArcSzamlalo {
    pub kesz: AtomicUsize,
    pub osszes: AtomicUsize,
}

pub fn indit_belyegkep_szalak(
    db: usize,
    munkak: &Arc<Sorban<Munka>>,
    ki: &Sender<Eredmeny>,
    gyorstar: &std::path::Path,
) {
    for _ in 0..db {
        let (munkak, ki, gyorstar) = (Arc::clone(munkak), ki.clone(), gyorstar.to_path_buf());
        std::thread::spawn(move || {
            prioritas::sajat_szal(prioritas::HATTER);
            loop {
                let Munka { id, foto, mutat } = munkak.kivesz();
                if !mutat && belyegkep::gyorstar_ut(&gyorstar, &foto).exists() {
                    continue;
                }
                let kep = belyegkep::betolt_vagy_keszit(&foto, &gyorstar)
                    .map_err(|e| naplo(&format!("belyegkep-hiba: {e}")))
                    .ok();
                if mutat && ki.send(Eredmeny::Belyegkep { id, kep }).is_err() {
                    break;
                }
            }
        });
    }
}

/// A megnyitott kép külön szálon dekódolódik, hogy ne várjon a háttérmunka mögött.
pub fn nagy_kep(id: usize, foto: Foto, ki: Sender<Eredmeny>) {
    std::thread::spawn(move || {
        let kep: Option<Pixelek> = belyegkep::dekodol(&foto.ut, 960)
            .map(|k| belyegkep::kicsinyit(k, 1024, 768).into())
            .map_err(|e| naplo(&format!("kepnyitasi hiba: {e}")))
            .ok();
        // ha a program közben bezárult, nincs kinek küldeni
        let _ = ki.send(Eredmeny::Nagy { id, kep });
    });
}

fn betolt_naplozva(modell: &std::path::Path) -> Detektor {
    let detektor = Detektor::betolt(modell);
    match detektor.helyettesito_oka() {
        Some(ok) => naplo(&format!("arckereses: HELYETTESITO munka ({ok})")),
        None => naplo("arckereses: YuNet (ONNX Runtime, 1 szal)"),
    }
    detektor
}

/// Egy szál a legalacsonyabb prioritással; a modellt az első képnél tölti be.
pub fn indit_arckereso(
    modell: PathBuf,
    naploja: PathBuf,
    szamlalo: Arc<ArcSzamlalo>,
) -> Sender<Foto> {
    let (ki, be) = mpsc::channel::<Foto>();
    std::thread::spawn(move || {
        prioritas::sajat_szal(prioritas::LEGALACSONYABB);
        let mut detektor: Option<Detektor> = None;
        let mut arcok = 0;
        let mut kezdet = Instant::now();
        for foto in be {
            let d = detektor.get_or_insert_with(|| {
                kezdet = Instant::now();
                betolt_naplozva(&modell)
            });
            match d.arcok_szama(&foto.ut) {
                Ok(n) => {
                    arcok += n;
                    if let Err(e) = arcnaplo::hozzafuz(&naploja, &foto, n) {
                        naplo(&format!("arcnaplo-hiba: {e}"));
                    }
                }
                Err(e) => naplo(&format!("arckeresesi hiba: {e}")),
            }
            let kesz = szamlalo.kesz.fetch_add(1, Ordering::SeqCst) + 1;
            if kesz == szamlalo.osszes.load(Ordering::SeqCst) {
                naplo(&format!(
                    "arckereses utolerte: {kesz} kep, {arcok} arc, {:.1} s ota",
                    kezdet.elapsed().as_secs_f64()
                ));
            }
        }
    });
    ki
}

/// Másodpercenként megnézi, van-e a gyökér alatt új, képet tartalmazó mappa.
pub fn indit_mappafigyelo(gyoker: PathBuf, mut ismert: HashSet<PathBuf>, ki: Sender<Eredmeny>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(1));
        for al in konyvtar::almappak(&gyoker) {
            if ismert.contains(&al) {
                continue;
            }
            let mappak = konyvtar::fa_beolvas(&al);
            if mappak.is_empty() {
                continue;
            }
            naplo(&format!(
                "uj mappa: {} ({} kep)",
                al.display(),
                mappak.iter().map(|m| m.fotok.len()).sum::<usize>()
            ));
            ismert.insert(al);
            if ki.send(Eredmeny::UjMappak(mappak)).is_err() {
                return;
            }
        }
    });
}
