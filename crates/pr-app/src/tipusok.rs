//! A mappafa és a képfeldolgozás közös, lemezolvasást nem végző adatai.

use std::path::PathBuf;

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
