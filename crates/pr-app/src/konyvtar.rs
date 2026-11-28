//! A gyökér alatti képmappák beolvasása.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

const KITERJESZTESEK: [&str; 3] = ["jpg", "jpeg", "png"];

#[derive(Debug, Clone, PartialEq)]
pub struct Foto {
    pub ut: PathBuf,
    pub meret: u64,
    pub mtime_ns: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Mappa {
    pub ut: PathBuf,
    pub nev: String,
    pub fotok: Vec<Foto>,
}

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

pub fn almappak(ut: &Path) -> Vec<PathBuf> {
    tartalom(ut).1
}

/// A mappa és alfái közül minden, amelyikben van kép.
pub fn fa_beolvas(ut: &Path) -> Vec<Mappa> {
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

    fn ideiglenes(nev: &str) -> Result<PathBuf, String> {
        let ut = std::env::temp_dir().join(format!("picarust-{nev}-{}", std::process::id()));
        if ut.exists() {
            std::fs::remove_dir_all(&ut).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&ut).map_err(|e| e.to_string())?;
        Ok(ut)
    }

    #[test]
    fn csak_a_kepet_tartalmazo_mappak_jonnek() -> Result<(), String> {
        let gy = ideiglenes("fa")?;
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
        let _ = std::fs::remove_dir_all(&gy);
        Ok(())
    }

    #[test]
    fn ures_es_nem_letezo_keptar_ures() -> Result<(), String> {
        let gy = ideiglenes("ures")?;
        assert!(fa_beolvas(&gy).is_empty());
        std::fs::remove_dir_all(&gy).map_err(|e| e.to_string())?;
        assert!(fa_beolvas(&gy).is_empty());
        Ok(())
    }

    #[test]
    fn ekezetes_szokoztartalmu_keptar_husz_kepet_beolvas_es_nem_ir_vissza() -> Result<(), String> {
        let gy = ideiglenes("ekezetes")?;
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
        let _ = std::fs::remove_dir_all(&gy);
        Ok(())
    }
}
