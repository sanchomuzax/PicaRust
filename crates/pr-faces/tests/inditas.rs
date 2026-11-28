use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use pr_faces::arc::Detektor;

fn ideiglenes_gyoker() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|gyoker| gyoker.join(".tmp"))
        .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())
}

#[test]
fn hianyzo_modell_ertheto_hibat_ad() -> Result<(), String> {
    let modell = ideiglenes_gyoker()?.join("nincs-ilyen-pr-faces-modell.onnx");
    let hiba = match Detektor::betolt(&modell) {
        Ok(_) => return Err("a hianyzo modell betoltese sikerult".to_string()),
        Err(hiba) => hiba,
    };
    if !hiba.contains("nincs modellfájl") {
        return Err(format!("a modellhiba nem ertheto: {hiba}"));
    }
    Ok(())
}

#[test]
fn hianyzo_dll_utvonal_ertheto_hibat_ad() -> Result<(), String> {
    const GYERMEK_TESZT: &str = "PICARUST_PR_FACES_DLL_GYERMEK";
    if std::env::var_os(GYERMEK_TESZT).is_some() {
        let mappa = ideiglenes_gyoker()?.join(format!("pr-faces-dll-{}", std::process::id()));
        fs::create_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
        let modell = mappa.join("modell.onnx");
        fs::write(&modell, b"tesztmodell").map_err(|hiba| hiba.to_string())?;
        let hiba = match Detektor::betolt(&modell) {
            Ok(_) => {
                return Err("a hianyzo ONNX Runtime DLL mellett a betoltes sikerult".to_string())
            }
            Err(hiba) => hiba,
        };
        if !hiba.contains("ORT_DYLIB_PATH") {
            return Err(format!(
                "a DLL-hiba nem emliti az ORT_DYLIB_PATH beallitast: {hiba}"
            ));
        }
        fs::remove_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
        return Ok(());
    }

    let dll = ideiglenes_gyoker()?.join(format!("nem-letezo-ort-{}.dll", std::process::id()));
    let binaris = std::env::current_exe().map_err(|hiba| hiba.to_string())?;
    let eredmeny = Command::new(binaris)
        .arg("--exact")
        .arg("hianyzo_dll_utvonal_ertheto_hibat_ad")
        .arg("--nocapture")
        .env(GYERMEK_TESZT, "1")
        .env("ORT_DYLIB_PATH", &dll)
        .output()
        .map_err(|hiba| hiba.to_string())?;
    if eredmeny.status.success() {
        return Ok(());
    }
    Err(format!(
        "a hianyzo DLL hibaja nem lett kezelve: {}{}",
        String::from_utf8_lossy(&eredmeny.stdout),
        String::from_utf8_lossy(&eredmeny.stderr)
    ))
}
