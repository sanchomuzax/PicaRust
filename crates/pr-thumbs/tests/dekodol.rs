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
    RgbImage::from_pixel(400, 300, Rgb([20, 40, 60]))
        .save_with_format(&ep, ImageFormat::Jpeg)
        .map_err(|hiba| hiba.to_string())?;
    let mut csonka_adat = fs::read(&ep).map_err(|hiba| hiba.to_string())?;
    if !csonka_adat.starts_with(&[0xff, 0xd8, 0xff]) || csonka_adat.len() < 8 {
        return Err("a generalt JPEG nem tartalmaz ervenyes fejlecet".to_string());
    }
    let scan_marker = csonka_adat
        .windows(2)
        .position(|bajtok| bajtok == [0xff, 0xda])
        .ok_or_else(|| "a generalt JPEG-bol hianyzik a beolvasasi fejlec".to_string())?;
    let scan_vege = scan_marker + 2;
    if scan_vege >= csonka_adat.len() {
        return Err("a generalt JPEG tul rovid a vagashoz".to_string());
    }
    csonka_adat.truncate(scan_vege);
    fs::write(&csonka, csonka_adat).map_err(|hiba| hiba.to_string())?;

    for ut in [&csonka, &ures] {
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

/// A képadat közepén megszakadt JPEG (félbemaradt másolás) nem hiba: a meglévő rész
/// bélyegképe elkészül, ahogy a PicasaPy-ban is (integrátori döntés, #15).
#[test]
fn a_kepadat_kozepen_megszakadt_jpeg_reszleges_belyegkepet_ad() -> Result<(), String> {
    let mappa = ideiglenes_gyoker()?.join(format!("pr-thumbs-kozep-{}", std::process::id()));
    if mappa.exists() {
        fs::remove_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
    }
    fs::create_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
    let eredmeny = kozepen_megszakadt_belyegkepe(&mappa);
    fs::remove_dir_all(&mappa).map_err(|hiba| hiba.to_string())?;
    let (szel, mag) = eredmeny?;
    if (szel, mag) != (144, 108) {
        return Err(format!("varatlan belyegkepmeret: {szel}x{mag}"));
    }
    Ok(())
}

fn kozepen_megszakadt_belyegkepe(mappa: &Path) -> Result<(u32, u32), String> {
    let ep = mappa.join("ep.jpg");
    RgbImage::from_pixel(400, 300, Rgb([20, 40, 60]))
        .save_with_format(&ep, ImageFormat::Jpeg)
        .map_err(|hiba| hiba.to_string())?;
    let adat = fs::read(&ep).map_err(|hiba| hiba.to_string())?;
    let scan_marker = adat
        .windows(2)
        .position(|bajtok| bajtok == [0xff, 0xda])
        .ok_or_else(|| "a generalt JPEG-bol hianyzik a beolvasasi fejlec".to_string())?;
    let kepadat_hossza = adat.len() - scan_marker;
    let vagas = scan_marker + kepadat_hossza / 2;
    let megszakadt = mappa.join("megszakadt.jpg");
    fs::write(&megszakadt, &adat[..vagas]).map_err(|hiba| hiba.to_string())?;
    let kep = betolt_vagy_keszit(&megszakadt, 17, 23, &mappa.join("cache"), |_| {})?;
    Ok((kep.szel as u32, kep.mag as u32))
}
