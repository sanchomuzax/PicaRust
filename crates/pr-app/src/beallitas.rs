//! Parancssori beállítások.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub struct Beallitas {
    pub gyoker: PathBuf,
    pub modell: PathBuf,
    pub gyorstar: PathBuf,
    pub onteszt_kep: Option<PathBuf>,
}

impl Beallitas {
    pub fn ertelmez<I: IntoIterator<Item = String>>(argumentumok: I) -> Result<Self, String> {
        let mut gyoker = kornyezeti_mappa("USERPROFILE").join("Pictures");
        let mut gyorstar = None;
        let mut modell = std::env::current_exe()
            .ok()
            .and_then(|e| e.parent().map(|m| m.join("yunet.onnx")))
            .unwrap_or_else(|| PathBuf::from("yunet.onnx"));
        let mut onteszt_kep = None;
        let mut sor = argumentumok.into_iter();
        while let Some(kapcsolo) = sor.next() {
            let ertek = sor
                .next()
                .map(PathBuf::from)
                .ok_or_else(|| format!("hianyzik a(z) {kapcsolo} erteke"))?;
            match kapcsolo.as_str() {
                "--gyoker" => gyoker = ertek,
                "--modell" => modell = ertek,
                "--gyorstar" => gyorstar = Some(ertek),
                "--onteszt" => onteszt_kep = Some(ertek),
                _ => return Err(format!("ismeretlen kapcsolo: {kapcsolo}")),
            }
        }
        let gyorstar = match gyorstar {
            Some(ut) => ut,
            None => alap_gyorstar()?,
        };
        Ok(Self {
            gyoker,
            modell,
            gyorstar,
            onteszt_kep,
        })
    }
}

fn alap_gyorstar() -> Result<PathBuf, String> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    let helyi_adat = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let xdg_adat = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    felhasznaloi_adatmappa(std::env::consts::OS, home, helyi_adat, xdg_adat)
}

fn felhasznaloi_adatmappa(
    operacios_rendszer: &str,
    home: Option<PathBuf>,
    helyi_adat: Option<PathBuf>,
    xdg_adat: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let gyoker = match operacios_rendszer {
        "windows" => helyi_adat.or_else(|| home.map(|ut| ut.join("AppData").join("Local"))),
        "macos" => home.map(|ut| ut.join("Library").join("Application Support")),
        "linux" | "android" => xdg_adat
            .filter(|ut| ut.is_absolute())
            .or_else(|| home.map(|ut| ut.join(".local").join("share"))),
        _ => home.map(|ut| ut.join(".local").join("share")),
    };
    match gyoker {
        Some(ut) => Ok(ut.join("PicaRustProba")),
        None => Err("nem található felhasználói adatmappa".to_string()),
    }
}

fn kornyezeti_mappa(nev: &str) -> PathBuf {
    std::env::var_os(nev)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tesztek {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_megadott_ertekek_ervenyesulnek() -> Result<(), String> {
        let b = Beallitas::ertelmez(args(&["--gyoker", "kepek", "--modell", "m.onnx"]))?;
        assert_eq!(b.gyoker, PathBuf::from("kepek"));
        assert_eq!(b.modell, PathBuf::from("m.onnx"));
        assert_eq!(b.onteszt_kep, None);
        Ok(())
    }

    #[test]
    fn a_hianyzo_ertek_es_az_ismeretlen_kapcsolo_hiba() {
        assert!(Beallitas::ertelmez(args(&["--gyoker"])).is_err());
        assert!(Beallitas::ertelmez(args(&["--valami", "x"])).is_err());
    }

    #[test]
    fn gyorsitotar_nem_a_keptarban() -> Result<(), String> {
        let beallitas = Beallitas::ertelmez(args(&["--gyoker", "Képek próba/árvíztűrő"]))?;
        let home = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .ok_or_else(|| "hiányzik a felhasználói mappa".to_string())?;
        let adatmappa = if cfg!(windows) {
            std::env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("AppData").join("Local"))
        } else if cfg!(target_os = "macos") {
            home.join("Library").join("Application Support")
        } else {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".local").join("share"))
        };
        assert!(
            beallitas.gyorstar.starts_with(&adatmappa),
            "a gyorsítótár nem a felhasználói adatmappa alatt van: {}",
            beallitas.gyorstar.display()
        );
        assert!(
            !beallitas.gyorstar.starts_with(&beallitas.gyoker),
            "a gyorsítótár a képtár alá került: {}",
            beallitas.gyorstar.display()
        );
        Ok(())
    }

    #[test]
    fn linuxon_az_xdg_adatmappat_hasznalja() -> Result<(), String> {
        let gyorstar = felhasznaloi_adatmappa(
            "linux",
            Some(PathBuf::from("felhasznalo")),
            None,
            Some(PathBuf::from("/adat")),
        )?;
        assert_eq!(gyorstar, PathBuf::from("/adat/PicaRustProba"));
        Ok(())
    }
}
