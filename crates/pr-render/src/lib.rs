//! Memóriában futó képeffekt-lánc. A csomag nem olvas és nem ír fájlt.
//!
//! Az `src/effektek/` mappába kizárólag effektet megvalósító `.rs` fájl kerüljön.
//! Minden fájl tegye közzé a kisbetűs `NEV` belső nevet, a pontos `LANC_NEV`
//! láncnevet és az `alkalmaz` függvényt. A fájlonkénti `PARAMETER_COUNT` a
//! láncolvasó a paraméterek számának ellenőrzéséhez használja. A `build.rs` a fájlnevekből hozza létre
//! a modulokat és az effektjegyzéket.

pub mod lanc;

use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq)]
pub enum Hiba {
    IsmeretlenEffekt(String),
    HibasLanc(String),
    HibasParameter { effekt: String, ok: String },
}

impl Display for Hiba {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IsmeretlenEffekt(nev) => write!(formatter, "Ismeretlen effekt: {nev}"),
            Self::HibasLanc(lanc) => write!(formatter, "Hibás effektlánc: {lanc}"),
            Self::HibasParameter { effekt, ok } => {
                write!(formatter, "Hibás paraméter, effekt: {effekt}: {ok}")
            }
        }
    }
}

impl std::error::Error for Hiba {}

pub(crate) type EffektAlkalmazo = fn(&image::RgbImage, &[String]) -> Result<image::RgbImage, Hiba>;

include!(concat!(env!("OUT_DIR"), "/effektek.rs"));

pub fn ismert_effektek() -> Vec<&'static str> {
    let mut nevek: Vec<_> = effektek::JEGYZEK
        .iter()
        .map(|(nev, _, _, _)| *nev)
        .collect();
    nevek.sort_unstable();
    nevek
}

pub(crate) fn effekt_nev_es_parameterek(lanc_nev: &str) -> Option<(&'static str, usize)> {
    effektek::JEGYZEK
        .iter()
        .find(|(_, kanonikus, _, _)| *kanonikus == lanc_nev)
        .map(|(nev, _, parameterek, _)| (*nev, *parameterek))
}

fn alkalmaz_lepest(kep: &image::RgbImage, lepes: &lanc::Lepes) -> Result<image::RgbImage, Hiba> {
    let alkalmaz = effektek::JEGYZEK
        .iter()
        .find(|(nev, _, _, _)| *nev == lepes.nev)
        .map(|(_, _, _, alkalmaz)| *alkalmaz)
        .ok_or_else(|| Hiba::IsmeretlenEffekt(lepes.nev.clone()))?;
    alkalmaz(kep, &lepes.parameterek)
}

/// A PicasaPy-val egyezően az érvényes láncelőtagot dolgozza fel.
///
/// Az első hibás vagy ismeretlen tag után az olvasó megáll. A hibás paraméterű
/// effektlépés kimarad, a következő érvényes lépés viszont továbbra is lefut.
pub fn alkalmaz(lanc: &str, kep: &image::RgbImage) -> Result<image::RgbImage, Hiba> {
    let ertelmezett = lanc::ertelmez(lanc);
    let mut eredmeny = kep.clone();
    for lepes in ertelmezett.lepesek {
        if let Ok(uj_kep) = alkalmaz_lepest(&eredmeny, &lepes) {
            eredmeny = uj_kep;
        }
    }
    Ok(eredmeny)
}

/// Szigorúan alkalmazza a teljes láncot, és hibát ad csonkolás vagy kihagyott lépés esetén.
pub fn alkalmaz_szigoruan(lanc: &str, kep: &image::RgbImage) -> Result<image::RgbImage, Hiba> {
    let ertelmezett = lanc::ertelmez(lanc);
    if let Some(ok) = ertelmezett.elvagva {
        return Err(Hiba::HibasLanc(ok));
    }

    let mut eredmeny = kep.clone();
    for lepes in ertelmezett.lepesek {
        eredmeny = alkalmaz_lepest(&eredmeny, &lepes)?;
    }
    Ok(eredmeny)
}

#[cfg(test)]
mod tesztek {
    use super::{alkalmaz, alkalmaz_szigoruan, effektek, Hiba};
    use std::collections::HashSet;

    fn tesztkep() -> image::RgbImage {
        image::RgbImage::from_pixel(1, 1, image::Rgb([81, 139, 204]))
    }

    #[test]
    fn effektjegyzek_nevei_nem_ures_kisbetusek_es_egyediek() {
        let mut nevek = HashSet::new();
        let mut lanc_nevek = HashSet::new();

        for (nev, lanc_nev, _, _) in effektek::JEGYZEK {
            assert!(!nev.is_empty(), "Üres NEV szerepel az effektjegyzékben");
            assert_eq!(*nev, nev.to_ascii_lowercase(), "A NEV nem kisbetűs: {nev}");
            assert!(nevek.insert(*nev), "Ismétlődő NEV: {nev}");
            assert!(
                lanc_nevek.insert(*lanc_nev),
                "Ismétlődő LANC_NEV: {lanc_nev}"
            );
            assert_eq!(
                *nev,
                lanc_nev.to_ascii_lowercase(),
                "A NEV és a LANC_NEV kisbetűs alakja eltér: {nev}, {lanc_nev}"
            );
        }
    }

    #[test]
    fn nem_veges_boost_parameternel_az_alaperteket_hasznalja() {
        let kep = tesztkep();
        let alap = alkalmaz("Boost=1;", &kep);

        for ertek in ["nan", "inf", "1e999"] {
            assert_eq!(alkalmaz(&format!("Boost=1,{ertek};"), &kep), alap);
        }
    }

    #[test]
    fn nagy_veges_boost_parameternel_visszateres_utan_is_ervenyes_a_kimenet() {
        let kep = tesztkep();
        let kapott = alkalmaz("Boost=1,1e300;", &kep);

        assert_eq!(kapott, alkalmaz("Boost=1,100;", &kep));
    }

    #[test]
    fn a_hibas_tag_utan_a_picasapy_szerinti_ervenyes_elotag_fut_le() {
        let kep = tesztkep();

        assert_eq!(
            alkalmaz("Boost=1,75;ismeretlen=1;Invert=1;", &kep),
            alkalmaz("Boost=1,75;", &kep)
        );
    }

    #[test]
    fn az_elso_mezo_erteket_nem_ellenorzi() {
        let kep = tesztkep();

        assert_eq!(alkalmaz("Boost=0,75;", &kep), alkalmaz("Boost=1,75;", &kep));
    }

    #[test]
    fn hibas_effektparameternel_kimarad_a_lepes_es_a_lanc_folytatodik() {
        let kep = tesztkep();

        assert_eq!(
            alkalmaz("Boost=1,hibas;Invert=1;", &kep),
            alkalmaz("Invert=1;", &kep)
        );
        assert!(matches!(
            alkalmaz_szigoruan("Boost=1,hibas;Invert=1;", &kep),
            Err(Hiba::HibasParameter { .. })
        ));
    }

    #[test]
    fn szigoru_alkalmazas_hibat_ad_elvagott_lancra() {
        assert!(matches!(
            alkalmaz_szigoruan("Boost=1;ismeretlen=1;Invert=1;", &tesztkep()),
            Err(Hiba::HibasLanc(_))
        ));
    }
}
