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
        let dll = std::env::var_os("ORT_DYLIB_PATH")
            .map(PathBuf::from)
            .ok_or_else(|| "hianyzik a gyermek ORT_DYLIB_PATH beallitasa".to_string())?;
        let mappa = dll
            .parent()
            .ok_or_else(|| "a teszt DLL-nek nincs szulomappaja".to_string())?;
        fs::create_dir_all(mappa).map_err(|hiba| hiba.to_string())?;
        fs::write(&dll, b"nem ervenyes dinamikus konyvtar").map_err(|hiba| hiba.to_string())?;
        let modell = mappa.join("modell.onnx");
        fs::write(&modell, b"tesztmodell").map_err(|hiba| hiba.to_string())?;
        let hiba = match Detektor::betolt(&modell) {
            Ok(_) => {
                return Err("a hianyzo ONNX Runtime DLL mellett a betoltes sikerult".to_string())
            }
            Err(hiba) => hiba,
        };
        if !hiba.contains("nem tölthető be")
            || hiba.contains("megadott DLL útvonal nem található")
            || !hiba.contains(&dll.to_string_lossy().to_string())
        {
            return Err(format!(
                "a DLL betoltesi hibaja nem jutott tul az elozetes utvonal-ellenorzesen: {hiba}"
            ));
        }
        return Ok(());
    }

    let dll = ideiglenes_gyoker()?
        .join(format!("pr-faces-dll-{}", std::process::id()))
        .join("nem-ervenyes-ort.dll");
    let binaris = std::env::current_exe().map_err(|hiba| hiba.to_string())?;
    let eredmeny = Command::new(binaris)
        .arg("--exact")
        .arg("hianyzo_dll_utvonal_ertheto_hibat_ad")
        .arg("--nocapture")
        .env(GYERMEK_TESZT, "1")
        .env("ORT_DYLIB_PATH", &dll)
        .output()
        .map_err(|hiba| hiba.to_string())?;
    if let Some(mappa) = dll.parent() {
        let _ = fs::remove_dir_all(mappa);
    }
    let kimenet = String::from_utf8_lossy(&eredmeny.stdout);
    let pontosan_egy_teszt = kimenet.lines().any(|sor| sor == "running 1 test");
    let a_teszt_atment = kimenet
        .lines()
        .any(|sor| sor == "test hianyzo_dll_utvonal_ertheto_hibat_ad ... ok");
    if eredmeny.status.success() && pontosan_egy_teszt && a_teszt_atment {
        return Ok(());
    }
    Err(format!(
        "a hianyzo DLL teszt nem pontosan egyszer es sikeresen futott (kilepes: {}): {}{}",
        eredmeny.status,
        kimenet,
        String::from_utf8_lossy(&eredmeny.stderr)
    ))
}
