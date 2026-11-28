//! A gyökér alatti képmappák beolvasása.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::tipusok::{Foto, Mappa};

const KITERJESZTESEK: [&str; 3] = ["jpg", "jpeg", "png"];

pub fn kep_e(ut: &Path) -> bool {
    ut.extension()
        .and_then(|k| k.to_str())
        .map(|k| KITERJESZTESEK.iter().any(|e| k.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

/// Egy mappa közvetlen képei és almappái, név szerint. A parancsikon-mappákat kihagyja.
fn tartalom(ut: &Path) -> (Vec<Foto>, Vec<PathBuf>) {
    let mut fotok = Vec::new();
    let mut almappak = Vec::new();
    let Ok(bejegyzesek) = std::fs::read_dir(ut) else {
        return (fotok, almappak);
    };
    for bejegyzes in bejegyzesek.flatten() {
        let Ok(fajta) = bejegyzes.file_type() else {
            continue;
        };
        let ut = bejegyzes.path();
        if fajta.is_dir() {
            almappak.push(ut);
        } else if fajta.is_file() && kep_e(&ut) {
            let Ok(adat) = bejegyzes.metadata() else {
                continue;
            };
            let mtime_ns = adat
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            fotok.push(Foto {
                ut,
                meret: adat.len(),
                mtime_ns,
            });
        }
    }
    fotok.sort_by(|a, b| a.ut.cmp(&b.ut));
    almappak.sort();
    (fotok, almappak)
}

pub(super) fn almappak(ut: &Path) -> Vec<PathBuf> {
    tartalom(ut).1
}

/// A mappa és alfái közül minden, amelyikben van kép.
pub(super) fn fa_beolvas(ut: &Path) -> Vec<Mappa> {
    let (fotok, almappak) = tartalom(ut);
    let mut ki = Vec::new();
    if !fotok.is_empty() {
        let nev = ut
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| ut.display().to_string());
        ki.push(Mappa {
            ut: ut.to_path_buf(),
            nev,
            fotok,
        });
    }
    for al in almappak {
        ki.extend(fa_beolvas(&al));
    }
    ki
}

#[cfg(test)]
mod tesztek {
    use super::*;

    struct TesztMappa(PathBuf);

    impl TesztMappa {
        fn uj(nev: &str) -> Result<Self, String> {
            let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
                .ancestors()
                .nth(2)
                .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
            let ut = munkafa
                .join(".tmp")
                .join(format!("picarust-{nev}-{}", std::process::id()));
            if ut.exists() {
                std::fs::remove_dir_all(&ut).map_err(|e| e.to_string())?;
            }
            std::fs::create_dir_all(&ut).map_err(|e| e.to_string())?;
            Ok(Self(ut))
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
    fn csak_a_kepet_tartalmazo_mappak_jonnek() -> Result<(), String> {
        let gy = TesztMappa::uj("fa")?;
        std::fs::create_dir_all(gy.join("b")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(gy.join("a")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(gy.join("ures")).map_err(|e| e.to_string())?;
        std::fs::write(gy.join("a/2.PNG"), b"x").map_err(|e| e.to_string())?;
        std::fs::write(gy.join("a/1.jpg"), b"x").map_err(|e| e.to_string())?;
        std::fs::write(gy.join("b/3.jpeg"), b"x").map_err(|e| e.to_string())?;
        std::fs::write(gy.join("b/nem.txt"), b"x").map_err(|e| e.to_string())?;
        let m = fa_beolvas(&gy);
        assert_eq!(
            m.iter().map(|m| m.nev.as_str()).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(m[0].fotok.len(), 2);
        assert!(m[0].fotok[0].ut.ends_with("1.jpg"));
        assert_eq!(m[1].fotok.len(), 1);
        assert_eq!(almappak(&gy).len(), 3);
        Ok(())
    }

    #[test]
    fn ures_es_nem_letezo_keptar_ures() -> Result<(), String> {
        let gy = TesztMappa::uj("ures")?;
        assert!(fa_beolvas(&gy).is_empty());
        std::fs::remove_dir_all(&*gy).map_err(|e| e.to_string())?;
        assert!(fa_beolvas(&gy).is_empty());
        Ok(())
    }

    #[test]
    fn ekezetes_szokoztartalmu_keptar_husz_kepet_beolvas_es_nem_ir_vissza() -> Result<(), String> {
        let gy = TesztMappa::uj("ekezetes")?;
        let album = gy.join("Képek próba").join("árvíztűrő");
        std::fs::create_dir_all(&album).map_err(|e| e.to_string())?;
        for index in 0..20 {
            std::fs::write(album.join(format!("foto-{index}.jpg")), b"")
                .map_err(|e| e.to_string())?;
        }
        let before = std::fs::read_dir(&album)
            .map_err(|e| e.to_string())?
            .map(|entry| {
                entry
                    .map_err(|e| e.to_string())
                    .map(|entry| entry.file_name())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mappak = fa_beolvas(&gy);
        let after = std::fs::read_dir(&album)
            .map_err(|e| e.to_string())?
            .map(|entry| {
                entry
                    .map_err(|e| e.to_string())
                    .map(|entry| entry.file_name())
            })
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(mappak.len(), 1);
        assert_eq!(mappak[0].nev, "árvíztűrő");
        assert_eq!(mappak[0].fotok.len(), 20);
        assert_eq!(before.len(), after.len());
        assert!(before.iter().all(|name| after.contains(name)));
        Ok(())
    }
}
