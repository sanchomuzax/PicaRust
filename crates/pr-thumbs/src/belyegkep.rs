//! Dekódolás, kicsinyítés és a bélyegképek lemezes gyorstára.

use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, RgbImage};

pub const MERET: u32 = 144;
const MINOSEG: u8 = 85;

/// Kész képpontok, amelyek szálak között átadhatók (a Slint képe nem az).
pub struct Pixelek {
    pub szel: u32,
    pub mag: u32,
    pub rgb: Vec<u8>,
}

impl From<RgbImage> for Pixelek {
    fn from(k: RgbImage) -> Self {
        let (szel, mag) = k.dimensions();
        Self {
            szel,
            mag,
            rgb: k.into_raw(),
        }
    }
}

/// A gyorstárfájl neve az útvonalból, a méretből és a módosítás idejéből.
pub fn gyorstar_ut(gyorstar: &Path, ut: &Path, meret: u64, mtime_ns: u128) -> PathBuf {
    let mut h = DefaultHasher::new();
    ut.hash(&mut h);
    meret.hash(&mut h);
    mtime_ns.hash(&mut h);
    gyorstar
        .join("belyegkep")
        .join(format!("{:016x}.jpg", h.finish()))
}

/// Dekódolás. JPEG-nél DCT-léptékezéssel: csak annyi képpont, hogy `legalabb` még meglegyen.
pub fn dekodol(ut: &Path, legalabb: u16) -> Result<RgbImage, String> {
    let jpeg = ut
        .extension()
        .and_then(|k| k.to_str())
        .map(|k| k.eq_ignore_ascii_case("jpg") || k.eq_ignore_ascii_case("jpeg"))
        .unwrap_or(false);
    if jpeg {
        if let Some(kep) = jpeg_kicsiben(ut, legalabb) {
            return Ok(kep);
        }
    }
    image::open(ut)
        .map(DynamicImage::into_rgb8)
        .map_err(|e| format!("{}: {e}", ut.display()))
}

/// `None`, ha nem sima RGB JPEG: azt az `image` crate dekódolja.
fn jpeg_kicsiben(ut: &Path, legalabb: u16) -> Option<RgbImage> {
    let mut dekoder = jpeg_decoder::Decoder::new(BufReader::new(File::open(ut).ok()?));
    let (szel, mag) = dekoder.scale(legalabb, legalabb).ok()?;
    let adat = dekoder.decode().ok()?;
    if adat.len() != usize::from(szel) * usize::from(mag) * 3 {
        return None;
    }
    RgbImage::from_raw(u32::from(szel), u32::from(mag), adat)
}

/// Arányosan befér a megadott dobozba; nagyítás nincs.
pub fn kicsinyit(kep: RgbImage, szel: u32, mag: u32) -> RgbImage {
    if kep.width() <= szel && kep.height() <= mag {
        return kep;
    }
    DynamicImage::ImageRgb8(kep)
        .thumbnail(szel, mag)
        .into_rgb8()
}

/// A bélyegkép a gyorstárból; ha nincs meg vagy sérült, elkészíti és elmenti.
pub fn betolt_vagy_keszit(
    ut: &Path,
    meret: u64,
    mtime_ns: u128,
    gyorstar: &Path,
    cache_hiba: impl FnOnce(&str),
) -> Result<Pixelek, String> {
    let cel = gyorstar_ut(gyorstar, ut, meret, mtime_ns);
    if let Ok(kep) = image::open(&cel) {
        return Ok(kep.into_rgb8().into());
    }
    let kicsi = kicsinyit(dekodol(ut, MERET as u16)?, MERET, MERET);
    if let Err(hiba) = ment(&cel, &kicsi) {
        cache_hiba(&format!("gyorstar-iras sikertelen: {hiba}"));
    }
    Ok(kicsi.into())
}

/// Ideiglenes fájlba ír, majd átnevez, hogy más szál ne lásson félkész képet.
fn ment(cel: &Path, kep: &RgbImage) -> Result<(), String> {
    static SORSZAM: AtomicUsize = AtomicUsize::new(0);
    let mappa = cel.parent().ok_or("nincs szulomappa")?;
    std::fs::create_dir_all(mappa).map_err(|e| e.to_string())?;
    let mut puffer = Vec::new();
    JpegEncoder::new_with_quality(&mut puffer, MINOSEG)
        .encode_image(kep)
        .map_err(|e| e.to_string())?;
    let ideiglenes = cel.with_extension(format!("{}.tmp", SORSZAM.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&ideiglenes, puffer).map_err(|e| e.to_string())?;
    std::fs::rename(&ideiglenes, cel).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tesztek {
    use super::*;

    type BelyegEredmeny = ((u32, u32), usize, bool);

    #[cfg(unix)]
    fn irasvedett_konyvtar(ut: &Path, irasvedett: bool) -> Result<(), String> {
        use std::os::unix::fs::PermissionsExt;

        let jog = if irasvedett { 0o555 } else { 0o755 };
        std::fs::set_permissions(ut, std::fs::Permissions::from_mode(jog))
            .map_err(|e| e.to_string())
    }

    #[cfg(not(unix))]
    fn irasvedett_konyvtar(_ut: &Path, _irasvedett: bool) -> Result<(), String> {
        Ok(())
    }

    #[test]
    fn a_kicsinyites_megtartja_az_aranyt_es_nem_nagyit() -> Result<(), String> {
        let nagy = kicsinyit(RgbImage::new(400, 300), 144, 144);
        assert_eq!(nagy.dimensions(), (144, 108));
        let kicsi = kicsinyit(RgbImage::new(20, 10), 144, 144);
        assert_eq!(kicsi.dimensions(), (20, 10));
        Ok(())
    }

    #[test]
    fn a_belyegkep_a_gyorstarba_kerul() -> Result<(), String> {
        let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
        let gy = munkafa
            .join(".tmp")
            .join(format!("picarust-belyeg-{}", std::process::id()));
        if gy.exists() {
            std::fs::remove_dir_all(&gy).map_err(|e| e.to_string())?;
        }
        let keptar = gy.join("keptar");
        let gyorstar = gy.join("felhasznaloi-adat");
        std::fs::create_dir_all(&keptar).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&gyorstar).map_err(|e| e.to_string())?;
        let mut fotok = Vec::new();
        for nev in ["a.jpg", "b.png"] {
            let ut = keptar.join(nev);
            RgbImage::new(1600, 1200)
                .save(&ut)
                .map_err(|e| e.to_string())?;
            fotok.push(ut);
        }
        irasvedett_konyvtar(&keptar, true)?;
        let eredmenyek = (|| -> Result<Vec<BelyegEredmeny>, String> {
            let mut eredmenyek = Vec::new();
            for ut in &fotok {
                let elso = betolt_vagy_keszit(ut, 1, 1, &gyorstar, |_| {})?;
                let gyorsitotarban = gyorstar_ut(&gyorstar, ut, 1, 1).is_file();
                let masodik = betolt_vagy_keszit(ut, 1, 1, &gyorstar, |_| {})?;
                eredmenyek.push(((elso.szel, elso.mag), masodik.rgb.len(), gyorsitotarban));
            }
            Ok(eredmenyek)
        })();
        let visszaallitas = irasvedett_konyvtar(&keptar, false);
        let eredmenyek = eredmenyek?;
        visszaallitas?;
        for ((meret, rgb_hossz, gyorsitotarban), nev) in eredmenyek.iter().zip(["a.jpg", "b.png"]) {
            assert_eq!(*meret, (144, 108), "{nev}");
            assert_eq!(*rgb_hossz, 144 * 108 * 3);
            assert!(*gyorsitotarban);
        }
        let keptar_fajlok = std::fs::read_dir(&keptar)
            .map_err(|e| e.to_string())?
            .map(|f| f.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(keptar_fajlok.len(), 2);
        let _ = std::fs::remove_dir_all(&gy);
        Ok(())
    }
}
