//! PicaRust képtárnéző alkalmazás Slint-felülettel.
//!
//! `picarust [--gyoker <mappa>] [--modell <onnx>] [--gyorstar <mappa>]`
//! `picarust --modell <onnx> --onteszt <kép>`: a modell betöltődik és talál-e arcot.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod beallitas;
mod elrendezes;
mod feladat;
mod felulet;
mod konyvtar;
mod munkasok;
mod prioritas;
mod racsmodell;

use std::path::Path;
use std::process::ExitCode;
use std::sync::OnceLock;
use std::time::Instant;

use pr_faces::arc;

#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "A Slint generált kódjában unwrap és panic hívások vannak."
)]
mod felulet_generalt {
    slint::include_modules!();
}

use felulet_generalt::{AppWindow, Cella, MappaElem, Sor};

/// Egy naplósor a hibakimenetre, az indulás óta eltelt idővel (a mérő fájlba irányítja).
pub fn naplo(szoveg: &str) {
    static KEZDET: OnceLock<Instant> = OnceLock::new();
    let t = KEZDET.get_or_init(Instant::now).elapsed().as_secs_f64();
    eprintln!("{t:9.3} {szoveg}");
}

fn main() -> ExitCode {
    naplo("PicaRust-proba indul");
    let beallitas = match beallitas::Beallitas::ertelmez(std::env::args().skip(1)) {
        Ok(b) => b,
        Err(e) => {
            naplo(&e);
            return ExitCode::from(2);
        }
    };
    if let Some(kep) = &beallitas.onteszt_kep {
        return onteszt(&beallitas.modell, kep);
    }
    match felulet::futtat(&beallitas) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            naplo(&format!("hiba: {e}"));
            ExitCode::FAILURE
        }
    }
}

/// A mérés előtti gyors ellenőrzés. Kilépési kód: 0 rendben, 3 nincs modell, 4 nincs arc.
fn onteszt(modell: &Path, kep: &Path) -> ExitCode {
    let mut detektor = match arc::Detektor::betolt(modell) {
        Ok(detektor) => detektor,
        Err(hiba) => {
            println!("onteszt: az arckereses nem indult el: {hiba}");
            return ExitCode::from(3);
        }
    };
    let t = Instant::now();
    match detektor.arcok_szama(kep) {
        Ok(n) => {
            println!("onteszt: {n} arc, {} ms", t.elapsed().as_millis());
            if n > 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(4)
            }
        }
        Err(e) => {
            println!("onteszt: hiba: {e}");
            ExitCode::FAILURE
        }
    }
}
