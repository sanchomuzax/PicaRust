//! A rács sorai: mappánként egy fejlécsor, utána a képsorok. Minden sor egyforma magas, ezért a
//! mappára ugrás helye pontosan kiszámolható.

pub const CELLA: f32 = 160.0;
pub const SOR_MAGAS: f32 = 160.0;
const MARGO: f32 = 8.0;

/// Egy mappa képei a közös képlistában.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tartomany {
    pub elso: usize,
    pub db: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SorTipus {
    Fejlec { mappa: usize },
    Kepek { elso: usize, db: usize },
}

pub fn oszlopok(szelesseg: f32) -> usize {
    (((szelesseg - MARGO) / CELLA).floor().max(1.0)) as usize
}

pub fn sorok(mappak: &[Tartomany], oszlop: usize) -> Vec<SorTipus> {
    let oszlop = oszlop.max(1);
    let mut ki = Vec::new();
    for (mappa, t) in mappak.iter().enumerate() {
        ki.push(SorTipus::Fejlec { mappa });
        for eltolas in (0..t.db).step_by(oszlop) {
            ki.push(SorTipus::Kepek {
                elso: t.elso + eltolas,
                db: oszlop.min(t.db - eltolas),
            });
        }
    }
    ki
}

pub fn mappa_sora(sorok: &[SorTipus], keresett: usize) -> Option<usize> {
    sorok
        .iter()
        .position(|s| matches!(s, SorTipus::Fejlec { mappa } if *mappa == keresett))
}

pub fn foto_sora(sorok: &[SorTipus], foto: usize) -> Option<usize> {
    sorok.iter().position(|s| match s {
        SorTipus::Kepek { elso, db } => foto >= *elso && foto < *elso + *db,
        SorTipus::Fejlec { .. } => false,
    })
}

/// A ListView `viewport-y` értéke, amellyel a `sor` kerül felülre (a lista végén visszafogva).
pub fn gorgetes_sorhoz(sor: usize, sor_db: usize, lathato: f32) -> f32 {
    let legtobb = (sor_db as f32 * SOR_MAGAS - lathato).max(0.0);
    -((sor as f32 * SOR_MAGAS).min(legtobb))
}

pub fn felso_sor(viewport_y: f32) -> usize {
    ((-viewport_y).max(0.0) / SOR_MAGAS).floor() as usize
}

/// Átrendezéskor (más oszlopszám) ugyanaz a tartalom maradjon felül.
pub fn horgony(regi: &[SorTipus], uj: &[SorTipus], felso: usize) -> usize {
    let cel = match regi.get(felso) {
        Some(SorTipus::Fejlec { mappa }) => mappa_sora(uj, *mappa),
        Some(SorTipus::Kepek { elso, .. }) => foto_sora(uj, *elso),
        None => None,
    };
    cel.unwrap_or(0)
}

#[cfg(test)]
mod tesztek {
    use super::*;

    const KET_MAPPA: [Tartomany; 2] = [Tartomany { elso: 0, db: 5 }, Tartomany { elso: 5, db: 2 }];

    #[test]
    fn a_sorok_fejlecbol_es_csonka_utolso_sorbol_allnak() {
        let s = sorok(&KET_MAPPA, 4);
        assert_eq!(
            s,
            vec![
                SorTipus::Fejlec { mappa: 0 },
                SorTipus::Kepek { elso: 0, db: 4 },
                SorTipus::Kepek { elso: 4, db: 1 },
                SorTipus::Fejlec { mappa: 1 },
                SorTipus::Kepek { elso: 5, db: 2 },
            ]
        );
        assert_eq!(mappa_sora(&s, 1), Some(3));
        assert_eq!(foto_sora(&s, 4), Some(2));
        assert_eq!(foto_sora(&s, 7), None);
    }

    #[test]
    fn oszlopszam_a_szelessegbol() {
        assert_eq!(oszlopok(794.0), 4);
        assert_eq!(oszlopok(0.0), 1);
        assert_eq!(oszlopok(1000.0), 6);
    }

    #[test]
    fn a_gorgetes_a_lista_vegen_megall() {
        assert_eq!(gorgetes_sorhoz(1, 10, 640.0), -160.0);
        assert_eq!(gorgetes_sorhoz(9, 10, 640.0), -960.0);
        assert_eq!(gorgetes_sorhoz(1, 2, 640.0), 0.0);
        assert_eq!(felso_sor(-330.0), 2);
        assert_eq!(felso_sor(5.0), 0);
    }

    #[test]
    fn atrendezeskor_a_tartalom_marad_felul() {
        let regi = sorok(&KET_MAPPA, 4);
        let uj = sorok(&KET_MAPPA, 2);
        // a régi 2. sor a 4. képpel kezdődik, az újban az a 3. sorban van
        assert_eq!(horgony(&regi, &uj, 2), 3);
        assert_eq!(horgony(&regi, &uj, 3), 4);
    }
}
