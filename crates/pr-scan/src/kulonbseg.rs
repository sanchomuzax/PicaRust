//! Két bejárás közötti különbségek megállapítása.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::Mappa;

/// Két bejárás közötti változások listája.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Valtozasok {
    pub uj_mappak: Vec<PathBuf>,
    pub megszunt_mappak: Vec<PathBuf>,
    pub uj_kepek: Vec<PathBuf>,
    pub megszunt_kepek: Vec<PathBuf>,
    pub modosult_kepek: Vec<PathBuf>,
}

impl Valtozasok {
    /// Igaz, ha mind az öt lista üres.
    #[must_use]
    pub fn ures(&self) -> bool {
        self.uj_mappak.is_empty()
            && self.megszunt_mappak.is_empty()
            && self.uj_kepek.is_empty()
            && self.megszunt_kepek.is_empty()
            && self.modosult_kepek.is_empty()
    }
}

/// Összeveti a régi és az új bejárás eredményét.
///
/// Tiszta függvény: kizárólag a kapott memóriabeli szeletekből dolgozik,
/// sem lemezolvasást, sem lemezírást nem végez.
#[must_use]
pub fn kulonbseg(regi: &[Mappa], uj: &[Mappa]) -> Valtozasok {
    let mut regi_mappak = BTreeSet::new();
    let mut regi_kepek = BTreeMap::new();
    for mappa in regi {
        regi_mappak.insert(&mappa.ut);
        for foto in &mappa.fotok {
            regi_kepek.insert(&foto.ut, (foto.meret, foto.mtime_ns));
        }
    }

    let mut uj_mappak_halmaz = BTreeSet::new();
    let mut uj_kepek_terkep = BTreeMap::new();
    for mappa in uj {
        uj_mappak_halmaz.insert(&mappa.ut);
        for foto in &mappa.fotok {
            uj_kepek_terkep.insert(&foto.ut, (foto.meret, foto.mtime_ns));
        }
    }

    let uj_mappak = uj_mappak_halmaz
        .difference(&regi_mappak)
        .copied()
        .cloned()
        .collect();

    let megszunt_mappak = regi_mappak
        .difference(&uj_mappak_halmaz)
        .copied()
        .cloned()
        .collect();

    let mut uj_kepek = Vec::new();
    let mut modosult_kepek = Vec::new();

    for (ut, (uj_meret, uj_mtime)) in &uj_kepek_terkep {
        match regi_kepek.get(ut) {
            None => {
                uj_kepek.push((*ut).clone());
            }
            Some((regi_meret, regi_mtime)) => {
                if regi_meret != uj_meret || regi_mtime != uj_mtime {
                    modosult_kepek.push((*ut).clone());
                }
            }
        }
    }

    let mut megszunt_kepek = Vec::new();
    for ut in regi_kepek.keys() {
        if !uj_kepek_terkep.contains_key(ut) {
            megszunt_kepek.push((*ut).clone());
        }
    }

    Valtozasok {
        uj_mappak,
        megszunt_mappak,
        uj_kepek,
        megszunt_kepek,
        modosult_kepek,
    }
}
