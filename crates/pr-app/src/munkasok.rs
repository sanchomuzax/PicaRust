//! Háttérszálak: bélyegképek (magonként egy), egy arckereső és egy mappafigyelő.

use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::feladat::Sorban;
use crate::{naplo, prioritas};
#[path = "konyvtar.rs"]
mod konyvtar;
pub use crate::tipusok::{Foto, Mappa};
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
    Indulo(Result<InduloAdat, String>),
    UjMappak(Vec<Mappa>),
}

pub struct InduloAdat {
    pub mappak: Vec<Mappa>,
    pub arc_kesz_kulcsok: HashSet<String>,
}

#[derive(Default)]
pub struct ArcSzamlalo {
    pub kesz: AtomicUsize,
    pub osszes: AtomicUsize,
}

/// A mappafa és az arcnapló lemezolvasását háttérszálon végzi.
pub fn indit_indulo_beolvasas(gyoker: PathBuf, arcnaplo_ut: PathBuf, ki: Sender<Eredmeny>) {
    indit_indulo_beolvasassal(ki, move || {
        let mappak = konyvtar::fa_beolvas(&gyoker);
        naplo(&format!(
            "beolvasas: {} mappa, {} kep",
            mappak.len(),
            mappak.iter().map(|m| m.fotok.len()).sum::<usize>()
        ));
        let arc_kesz_kulcsok = arcnaplo_beolvas(&arcnaplo_ut);
        Ok(InduloAdat {
            mappak,
            arc_kesz_kulcsok,
        })
    });
}

fn arcnaplo_beolvas(ut: &Path) -> HashSet<String> {
    std::fs::read_to_string(ut)
        .map(|tartalom| {
            tartalom
                .lines()
                .filter_map(|sor| sor.rsplit_once('\t').map(|(kulcs, _)| kulcs.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn arckeresesbol_kimarad(foto: &Foto, feldolgozott: &HashSet<String>) -> bool {
    feldolgozott.contains(&arcnaplo::kulcs(&foto.ut, foto.meret, foto.mtime_ns))
}

fn indit_indulo_beolvasassal<F>(ki: Sender<Eredmeny>, olvaso: F)
where
    F: FnOnce() -> Result<InduloAdat, String> + Send + 'static,
{
    std::thread::spawn(move || {
        let eredmeny = catch_unwind(AssertUnwindSafe(|| {
            prioritas::sajat_szal(prioritas::HATTER);
            olvaso()
        }))
        .unwrap_or_else(|_| Err("Az induló képtár beolvasása váratlanul leállt.".to_string()));
        let _ = ki.send(Eredmeny::Indulo(eredmeny));
    });
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

    struct ArcnaploTesztfajl(PathBuf);

    impl ArcnaploTesztfajl {
        fn uj(nev: &str) -> Result<Self, String> {
            let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
            let ut = munkafa.join(".tmp").join(format!(
                "picarust-arcnaplo-{nev}-{}.tsv",
                std::process::id()
            ));
            std::fs::create_dir_all(ut.parent().unwrap_or(munkafa))
                .map_err(|hiba| hiba.to_string())?;
            let _ = std::fs::remove_file(&ut);
            Ok(Self(ut))
        }
    }

    impl std::ops::Deref for ArcnaploTesztfajl {
        type Target = Path;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Drop for ArcnaploTesztfajl {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    struct TesztMappa(PathBuf);

    impl TesztMappa {
        fn uj(nev: &str) -> Result<Self, String> {
            let gyoker = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?
                .join(".tmp")
                .join(format!("picarust-{nev}-{}", std::process::id()));
            if gyoker.exists() {
                std::fs::remove_dir_all(&gyoker).map_err(|hiba| hiba.to_string())?;
            }
            std::fs::create_dir_all(&gyoker).map_err(|hiba| hiba.to_string())?;
            Ok(Self(gyoker))
        }
    }

    impl std::ops::Deref for TesztMappa {
        type Target = Path;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Drop for TesztMappa {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn indulo_mappafa_es_arcnaplo_eredmenye_uzenetben_erkezik() -> Result<(), String> {
        let gyoker = TesztMappa::uj("indulo")?;
        let album = gyoker.join("album");
        std::fs::create_dir_all(&album).map_err(|hiba| hiba.to_string())?;
        std::fs::write(album.join("foto.jpg"), b"teszt").map_err(|hiba| hiba.to_string())?;
        let arcnaplo_ut = gyoker.join("arcok.tsv");
        std::fs::write(&arcnaplo_ut, "felismert\t0\n").map_err(|hiba| hiba.to_string())?;

        let (ki, be) = mpsc::channel();
        indit_indulo_beolvasas(gyoker.to_path_buf(), arcnaplo_ut, ki);
        let eredmeny = be
            .recv_timeout(Duration::from_secs(3))
            .map_err(|hiba| format!("nem erkezett meg az indulo eredmeny: {hiba}"))?;
        let Eredmeny::Indulo(Ok(InduloAdat {
            mappak,
            arc_kesz_kulcsok,
        })) = eredmeny
        else {
            return Err("nem az indulo mappak es arcnaplo erkezett meg".to_string());
        };

        assert_eq!(mappak.len(), 1);
        assert_eq!(mappak[0].nev, "album");
        assert!(arc_kesz_kulcsok.contains("felismert"));
        Ok(())
    }

    #[test]
    fn arcnaplo_beolvas_hianyzo_fajlnal_ures_halmaz() -> Result<(), String> {
        let ut = ArcnaploTesztfajl::uj("hianyzik")?;

        assert!(!ut.exists());
        assert!(arcnaplo_beolvas(&ut).is_empty());
        Ok(())
    }

    #[test]
    fn arcnaplo_beolvas_kihagyja_a_tabulator_nelkuli_sort() -> Result<(), String> {
        let ut = ArcnaploTesztfajl::uj("sorok")?;
        let elso = arcnaplo::kulcs(Path::new("album/elso.jpg"), 12, 34);
        let masodik = arcnaplo::kulcs(Path::new("album/masodik.jpg"), 56, 78);
        std::fs::write(&*ut, format!("hibas sor\n{elso}\t2\n{masodik}\t0\n"))
            .map_err(|hiba| hiba.to_string())?;

        assert_eq!(arcnaplo_beolvas(&ut), HashSet::from([elso, masodik]));
        Ok(())
    }

    #[test]
    fn arcnaplo_hozzafuz_es_beolvas_korforgasa_megorizi_a_kulcsot() -> Result<(), String> {
        let ut = ArcnaploTesztfajl::uj("korforgas")?;
        let kep_ut = Path::new("album/foto.jpg");
        let vart = arcnaplo::kulcs(kep_ut, 123, 456);
        arcnaplo::hozzafuz(&ut, kep_ut, 123, 456, 3).map_err(|hiba| hiba.to_string())?;

        assert_eq!(arcnaplo_beolvas(&ut), HashSet::from([vart]));
        Ok(())
    }

    #[test]
    fn indulo_inditoja_visszater_a_beolvasas_befejezese_elott() -> Result<(), String> {
        let (elindult_ki, elindult_be) = mpsc::channel();
        let (folytatas_ki, folytatas_be) = mpsc::channel();
        let (indito_visszatert_ki, indito_visszatert_be) = mpsc::channel();
        let (eredmeny_ki, eredmeny_be) = mpsc::channel();

        let indito = std::thread::spawn(move || {
            indit_indulo_beolvasassal(eredmeny_ki, move || {
                let _ = elindult_ki.send(());
                let _ = folytatas_be.recv();
                Ok(InduloAdat {
                    mappak: Vec::new(),
                    arc_kesz_kulcsok: HashSet::new(),
                })
            });
            let _ = indito_visszatert_ki.send(());
        });

        elindult_be
            .recv_timeout(Duration::from_secs(3))
            .map_err(|hiba| format!("a cserelt beolvaso nem indult el: {hiba}"))?;
        let indito_visszatert = indito_visszatert_be
            .recv_timeout(Duration::from_secs(10))
            .is_ok();
        let _ = folytatas_ki.send(());
        indito
            .join()
            .map_err(|_| "az indito szal panikolt".to_string())?;
        assert!(
            indito_visszatert,
            "az indito hivas a beolvasas befejezese elott terjen vissza"
        );
        assert!(matches!(
            eredmeny_be.recv_timeout(Duration::from_secs(3)),
            Ok(Eredmeny::Indulo(Ok(_)))
        ));
        Ok(())
    }

    #[test]
    fn mar_feldolgozott_kep_kimarad_az_arckeresesbol() {
        let foto = Foto {
            ut: PathBuf::from("album/foto.jpg"),
            meret: 123,
            mtime_ns: 456,
        };
        let feldolgozott = HashSet::from([arcnaplo::kulcs(&foto.ut, foto.meret, foto.mtime_ns)]);

        assert!(arckeresesbol_kimarad(&foto, &feldolgozott));
        assert!(!arckeresesbol_kimarad(
            &Foto {
                ut: PathBuf::from("album/uj.jpg"),
                ..foto
            },
            &feldolgozott
        ));
    }

    #[test]
    fn indulo_hibanal_es_panikhataron_is_uzenetet_kuld() -> Result<(), String> {
        let (ki, be) = mpsc::channel();
        indit_indulo_beolvasassal(ki, || Err("teszt beolvasasi hiba".to_string()));
        assert!(matches!(
            be.recv_timeout(Duration::from_secs(3)),
            Ok(Eredmeny::Indulo(Err(hiba))) if hiba == "teszt beolvasasi hiba"
        ));

        let (ki, be) = mpsc::channel();
        indit_indulo_beolvasassal(ki, || -> Result<InduloAdat, String> {
            std::panic::resume_unwind(Box::new("tesztpanik"))
        });
        assert!(matches!(
            be.recv_timeout(Duration::from_secs(3)),
            Ok(Eredmeny::Indulo(Err(_)))
        ));
        Ok(())
    }

    #[test]
    fn hianyzo_modellnel_a_hattermunka_kereses_nelkul_folytatodik() -> Result<(), String> {
        let gyoker = TesztMappa::uj("arckereso")?;
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
        assert!(
            !naploja.exists(),
            "a hianyzo modell mellett arckeresesi naplo keszult"
        );
        Ok(())
    }
}
