//! Háttérszálak: bélyegképek (magonként egy), egy arckereső és egy mappafigyelő.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::feladat::Sorban;
use crate::konyvtar::{self, Foto, Mappa};
use crate::{naplo, prioritas};
use pr_faces::arc::Detektor;
use pr_faces::arcnaplo;
use pr_thumbs::belyegkep::{self, Pixelek};

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
                if !mutat
                    && belyegkep::gyorstar_ut(&gyorstar, &foto.ut, foto.meret, foto.mtime_ns)
                        .exists()
                {
                    continue;
                }
                let kep = belyegkep::betolt_vagy_keszit(
                    &foto.ut,
                    foto.meret,
                    foto.mtime_ns,
                    &gyorstar,
                    naplo,
                )
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

fn betolt_naplozva(modell: &std::path::Path) -> Result<Detektor, String> {
    match Detektor::betolt(modell) {
        Ok(detektor) => {
            naplo("arckereses: YuNet (ONNX Runtime, 1 szal)");
            Ok(detektor)
        }
        Err(hiba) => {
            naplo(&format!("arckereses kikapcsolva: {hiba}"));
            Err(hiba)
        }
    }
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
        let mut detektor: Option<Result<Detektor, String>> = None;
        let mut arcok = 0;
        let mut kezdet = Instant::now();
        for foto in be {
            let d = detektor.get_or_insert_with(|| {
                kezdet = Instant::now();
                betolt_naplozva(&modell)
            });
            if let Ok(d) = d {
                match d.arcok_szama(&foto.ut) {
                    Ok(n) => {
                        arcok += n;
                        if let Err(e) =
                            arcnaplo::hozzafuz(&naploja, &foto.ut, foto.meret, foto.mtime_ns, n)
                        {
                            naplo(&format!("arcnaplo-hiba: {e}"));
                        }
                    }
                    Err(e) => naplo(&format!("arckeresesi hiba: {e}")),
                }
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

#[cfg(test)]
mod tesztek {
    use super::*;
    use image::{ImageFormat, Rgb, RgbImage};
    use std::path::Path;
    use std::time::Duration;

    struct IdeiglenesMappa(PathBuf);

    impl Drop for IdeiglenesMappa {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn hianyzo_modellnel_a_hattermunka_kereses_nelkul_folytatodik() -> Result<(), String> {
        let gyoker = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?
            .join(".tmp")
            .join(format!("pr-app-arckereso-{}", std::process::id()));
        let _takaritas = IdeiglenesMappa(gyoker.clone());
        if gyoker.exists() {
            std::fs::remove_dir_all(&gyoker).map_err(|hiba| hiba.to_string())?;
        }
        std::fs::create_dir_all(&gyoker).map_err(|hiba| hiba.to_string())?;
        let kep_ut = gyoker.join("valodi-tesztkep.jpg");
        RgbImage::from_pixel(400, 300, Rgb([20, 40, 60]))
            .save_with_format(&kep_ut, ImageFormat::Jpeg)
            .map_err(|hiba| hiba.to_string())?;
        let foto = Foto {
            meret: std::fs::metadata(&kep_ut)
                .map_err(|hiba| hiba.to_string())?
                .len(),
            ut: kep_ut,
            mtime_ns: 0,
        };
        let gyorstar = gyoker.join("gyorstar");
        let belyeg_ut = belyegkep::gyorstar_ut(&gyorstar, &foto.ut, foto.meret, foto.mtime_ns);
        let azonosito = std::process::id();
        let naploja = gyoker.join(format!("arcok-{azonosito}.tsv"));

        let (belyeg_ki, belyeg_be) = mpsc::channel();
        let sor = Arc::new(Sorban::uj());
        sor.hatter([Munka {
            id: 41,
            foto: foto.clone(),
            mutat: true,
        }]);
        indit_belyegkep_szalak(1, &sor, &belyeg_ki, &gyorstar);

        let szamlalo = Arc::new(ArcSzamlalo::default());
        szamlalo.osszes.store(1, Ordering::SeqCst);
        let kuldo = indit_arckereso(
            gyoker.join(format!("nem-letezo-modell-{azonosito}.onnx")),
            naploja.clone(),
            Arc::clone(&szamlalo),
        );
        kuldo.send(foto).map_err(|hiba| hiba.to_string())?;
        drop(kuldo);

        let belyeg = belyeg_be
            .recv_timeout(Duration::from_secs(3))
            .map_err(|hiba| format!("a belyegkep-munkas nem adott eredmenyt: {hiba}"))?;
        match belyeg {
            Eredmeny::Belyegkep {
                id: 41,
                kep: Some(kep),
            } if (kep.szel, kep.mag) == (144, 108) => {}
            Eredmeny::Belyegkep { kep: None, .. } => {
                return Err("a belyegkep-munkas nem keszitett kepet".to_string())
            }
            Eredmeny::Belyegkep { kep: Some(kep), .. } => {
                return Err(format!("varatlan belyegkepmeret: {}x{}", kep.szel, kep.mag))
            }
            _ => return Err("a belyegkep-munkas varatlan eredmenyt adott".to_string()),
        }
        if !belyeg_ut.is_file() {
            return Err("a belyegkep-munkas nem mentette a gyorstarba a kepet".to_string());
        }

        let hatarido = Instant::now() + Duration::from_secs(3);
        while szamlalo.kesz.load(Ordering::SeqCst) == 0 && Instant::now() < hatarido {
            std::thread::sleep(Duration::from_millis(10));
        }
        if szamlalo.kesz.load(Ordering::SeqCst) != 1 {
            return Err("a hianyzo modell melletti munka nem fejezodott be".to_string());
        }
        if naploja.exists() {
            return Err("a hianyzo modell mellett arckeresesi naplo keszult".to_string());
        }
        Ok(())
    }
}
