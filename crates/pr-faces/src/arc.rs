//! Arckeresés a PicasaPy YuNet-modelljével, ONNX Runtime-mal.
//!
//! Az előkészítés és a dekódolás az OpenCV `FaceDetectorYN`-jét követi (a PicasaPy azt hívja):
//! 960 képpontos hosszabb oldal, BGR, 0-255, 32 többszörösére nullákkal kiegészítve. A PicasaPy
//! üres eredmény utáni második és harmadik menete, valamint az SFace-vektorok itt nincsenek meg.
//! A modell bemenetét a yunet_dinamikus.py teszi tetszőleges méretűvé.

use image::RgbImage;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;

use std::any::Any;
use std::path::Path;

use pr_thumbs::belyegkep;

const HOSSZABB_OLDAL: u32 = 960;
const PONTSZAM_KUSZOB: f32 = 0.9;
const NMS_KUSZOB: f32 = 0.3;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Jelolt {
    pub x: f32,
    pub y: f32,
    pub szel: f32,
    pub mag: f32,
    pub pontszam: f32,
}

pub struct Bemenet {
    pub szel: usize,
    pub mag: usize,
    pub adat: Vec<f32>,
}

pub enum Detektor {
    YuNet(Session),
}

impl Detektor {
    pub fn betolt(modell: &Path) -> Result<Self, String> {
        munkamenet(modell).map(Detektor::YuNet)
    }

    pub fn arcok_szama(&mut self, kep: &Path) -> Result<usize, String> {
        let kicsi = belyegkep::kicsinyit(
            belyegkep::dekodol(kep, HOSSZABB_OLDAL as u16)?,
            HOSSZABB_OLDAL,
            HOSSZABB_OLDAL,
        );
        let bemenet = tensor_adat(&kicsi);
        match self {
            Detektor::YuNet(munkamenet) => Ok(kovetkeztet(munkamenet, bemenet)?.len()),
        }
    }
}

fn szoveg<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Egy szálon fut (a hívó szálán), így annak alacsony prioritása érvényes rá.
fn munkamenet(modell: &Path) -> Result<Session, String> {
    if !modell.is_file() {
        return Err(format!("nincs modellfájl: {}", modell.display()));
    }
    if let Ok(dll) = std::env::var("ORT_DYLIB_PATH") {
        let dll_ut = Path::new(&dll);
        if dll_ut.components().count() > 1 && !dll_ut.is_file() {
            return Err(ort_hiba(format!(
                "a megadott DLL útvonal nem található: {dll}"
            )));
        }
    }
    let betoltes = std::panic::catch_unwind(|| {
        Session::builder()
            .map_err(szoveg)?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(szoveg)?
            .with_intra_threads(1)
            .map_err(szoveg)?
            .commit_from_file(modell)
            .map_err(szoveg)
    });
    match betoltes {
        Ok(Ok(munkamenet)) => Ok(munkamenet),
        Ok(Err(hiba)) => Err(format!(
            "az ONNX Runtime munkamenet nem tölthető be: {hiba}"
        )),
        Err(hiba) => Err(ort_hiba(panik_szoveg(hiba.as_ref()).to_string())),
    }
}

fn panik_szoveg(hiba: &(dyn Any + Send)) -> &str {
    if let Some(szoveg) = hiba.downcast_ref::<String>() {
        szoveg.as_str()
    } else if let Some(szoveg) = hiba.downcast_ref::<&str>() {
        szoveg
    } else {
        "ismeretlen ONNX Runtime hiba"
    }
}

fn ort_hiba<E: std::fmt::Display>(hiba: E) -> String {
    let dll = match std::env::var("ORT_DYLIB_PATH") {
        Ok(ut) => ut,
        Err(_) => "az alapértelmezett hely".to_string(),
    };
    format!(
        "az ONNX Runtime DLL nem tölthető be ({dll}); ellenőrizd az ORT_DYLIB_PATH beállítását: {hiba}"
    )
}

/// BGR sorrendű NCHW-tömb, jobbra és lent 32 többszörösére nullákkal kiegészítve.
pub fn tensor_adat(kep: &RgbImage) -> Bemenet {
    let szel = (kep.width() as usize).div_ceil(32) * 32;
    let mag = (kep.height() as usize).div_ceil(32) * 32;
    let sik = szel * mag;
    let mut adat = vec![0f32; 3 * sik];
    for (x, y, p) in kep.enumerate_pixels() {
        let i = y as usize * szel + x as usize;
        adat[i] = f32::from(p[2]);
        adat[sik + i] = f32::from(p[1]);
        adat[2 * sik + i] = f32::from(p[0]);
    }
    Bemenet { szel, mag, adat }
}

fn kovetkeztet(munkamenet: &mut Session, bemenet: Bemenet) -> Result<Vec<Jelolt>, String> {
    let Bemenet { szel, mag, adat } = bemenet;
    let tensor =
        Tensor::from_array(([1usize, 3, mag, szel], adat.into_boxed_slice())).map_err(szoveg)?;
    let kimenet = munkamenet
        .run(ort::inputs!["input" => tensor])
        .map_err(szoveg)?;
    let mut jeloltek = Vec::new();
    for lepes in [8usize, 16, 32] {
        let (_, cls) = kimenet[format!("cls_{lepes}").as_str()]
            .try_extract_tensor::<f32>()
            .map_err(szoveg)?;
        let (_, obj) = kimenet[format!("obj_{lepes}").as_str()]
            .try_extract_tensor::<f32>()
            .map_err(szoveg)?;
        let (_, bbox) = kimenet[format!("bbox_{lepes}").as_str()]
            .try_extract_tensor::<f32>()
            .map_err(szoveg)?;
        jeloltek.extend(dekodol_szint(cls, obj, bbox, szel / lepes, lepes));
    }
    Ok(nms(jeloltek, NMS_KUSZOB))
}

/// Egy lépésköz kimenetéből a küszöb feletti keretek, az OpenCV képlete szerint.
pub fn dekodol_szint(
    cls: &[f32],
    obj: &[f32],
    bbox: &[f32],
    oszlop: usize,
    lepes: usize,
) -> Vec<Jelolt> {
    let l = lepes as f32;
    cls.iter()
        .zip(obj)
        .enumerate()
        .filter_map(|(i, (c, o))| {
            let pontszam = (c.clamp(0.0, 1.0) * o.clamp(0.0, 1.0)).sqrt();
            if pontszam < PONTSZAM_KUSZOB {
                return None;
            }
            let b = bbox.get(4 * i..4 * i + 4)?;
            let cx = ((i % oszlop) as f32 + b[0]) * l;
            let cy = ((i / oszlop) as f32 + b[1]) * l;
            let (szel, mag) = (b[2].exp() * l, b[3].exp() * l);
            Some(Jelolt {
                x: cx - szel / 2.0,
                y: cy - mag / 2.0,
                szel,
                mag,
                pontszam,
            })
        })
        .collect()
}

pub fn nms(mut jeloltek: Vec<Jelolt>, kuszob: f32) -> Vec<Jelolt> {
    jeloltek.sort_by(|a, b| b.pontszam.total_cmp(&a.pontszam));
    let mut marad: Vec<Jelolt> = Vec::new();
    for j in jeloltek {
        if marad.iter().all(|m| iou(m, &j) <= kuszob) {
            marad.push(j);
        }
    }
    marad
}

fn iou(a: &Jelolt, b: &Jelolt) -> f32 {
    let w = (a.x + a.szel).min(b.x + b.szel) - a.x.max(b.x);
    let h = (a.y + a.mag).min(b.y + b.mag) - a.y.max(b.y);
    if w <= 0.0 || h <= 0.0 {
        return 0.0;
    }
    let metszet = w * h;
    metszet / (a.szel * a.mag + b.szel * b.mag - metszet)
}

#[cfg(test)]
mod tesztek {
    use super::*;

    #[test]
    fn a_tensor_bgr_es_kiegeszitett() -> Result<(), String> {
        let mut kep = RgbImage::new(40, 30);
        kep.put_pixel(1, 0, image::Rgb([10, 20, 30]));
        let b = tensor_adat(&kep);
        assert_eq!((b.szel, b.mag), (64, 32));
        let sik = 64 * 32;
        assert_eq!(
            [b.adat[1], b.adat[sik + 1], b.adat[2 * sik + 1]],
            [30.0, 20.0, 10.0]
        );
        Ok(())
    }

    #[test]
    fn a_dekodolas_a_cella_helyere_teszi_a_keretet() -> Result<(), String> {
        let cls = [0.0, 0.0, 0.0, 1.0];
        let obj = [1.0, 1.0, 1.0, 0.9];
        let bbox = [0.0; 16];
        let j = dekodol_szint(&cls, &obj, &bbox, 2, 8);
        assert_eq!(j.len(), 1);
        // a 3. cella (1. oszlop, 1. sor): középpont 8;8, oldal 8
        assert_eq!((j[0].x, j[0].y, j[0].szel, j[0].mag), (4.0, 4.0, 8.0, 8.0));
        Ok(())
    }

    #[test]
    fn az_atfedo_keretekbol_a_legerosebb_marad() -> Result<(), String> {
        let k = |x, pontszam| Jelolt {
            x,
            y: 0.0,
            szel: 10.0,
            mag: 10.0,
            pontszam,
        };
        let marad = nms(vec![k(1.0, 0.91), k(0.0, 0.99), k(50.0, 0.95)], 0.3);
        assert_eq!(marad.iter().map(|j| j.x).collect::<Vec<_>>(), [0.0, 50.0]);
        Ok(())
    }

    #[test]
    fn hianyzo_modellnel_hiba_ter_vissza() -> Result<(), String> {
        let hiba = match Detektor::betolt(Path::new("nincs-ilyen-modell.onnx")) {
            Ok(_) => return Err("a hianyzo modell betoltese sikerult".to_string()),
            Err(hiba) => hiba,
        };
        assert!(hiba.contains("nincs modellfájl"));
        Ok(())
    }

    /// A próbaprogram eredeti tesztje, a nevét megtartva: modell nélkül a detektor nem áll
    /// össze. Régen helyettesítő munkát adott, most hibát ad, és a program arckeresés nélkül
    /// megy tovább (ezt a `pr-app` háttérmunkás-tesztje védi).
    #[test]
    fn modell_nelkul_helyettesito_munka() {
        assert!(Detektor::betolt(Path::new("nincs-ilyen-modell.onnx")).is_err());
    }
}
