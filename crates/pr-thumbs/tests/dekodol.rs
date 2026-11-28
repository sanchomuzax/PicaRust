use std::fs;
use std::path::{Path, PathBuf};

use image::{ImageFormat, Rgb, RgbImage};
use pr_thumbs::belyegkep::{betolt_vagy_keszit, dekodol};

fn ideiglenes_gyoker() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|gyoker| gyoker.join(".tmp"))
        .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())
}

#[test]
fn a_hibas_jpegek_hibat_adnak_es_a_melletti_ep_kep_belyegkepe_elkeszul() -> Result<(), String> {
    let mappa = ideiglenes_gyoker()?.join(format!("pr-thumbs-{}", std::process::id()));
    if mappa.exists() {
        fs::remove_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
    }
    fs::create_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;

    let ures = mappa.join("ures.jpg");
    let csonka = mappa.join("csonka.jpg");
    let ep = mappa.join("ep.jpg");
    fs::write(&ures, []).map_err(|hiba| hiba.to_string())?;
    fs::write(&csonka, [0xff, 0xd8, 0xff]).map_err(|hiba| hiba.to_string())?;
    RgbImage::from_pixel(400, 300, Rgb([20, 40, 60]))
        .save_with_format(&ep, ImageFormat::Jpeg)
        .map_err(|hiba| hiba.to_string())?;

    for ut in [&ures, &csonka] {
        if dekodol(ut, 144).is_ok() {
            return Err(format!(
                "a hibas JPEG dekodolasa sikerult: {}",
                ut.display()
            ));
        }
    }

    let kep = betolt_vagy_keszit(&ep, 17, 23, &mappa.join("cache"), |_| {})?;
    if (kep.szel, kep.mag) != (144, 108) {
        return Err(format!("varatlan belyegkepmeret: {}x{}", kep.szel, kep.mag));
    }

    fs::remove_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
    Ok(())
}
