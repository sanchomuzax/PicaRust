//! A képtár bejárása és a változások észlelése.
//!
//! Ez a csomag olvassa a lemezről a mappák és a képfájlok listáját, és megmondja, mi változott két
//! bejárás között. A képtárba nem ír. A felülettől nem függ.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

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
