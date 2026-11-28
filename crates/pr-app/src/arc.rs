//! Arckeresés a PicasaPy YuNet-modelljével, ONNX Runtime-mal.
//!
//! Az előkészítés és a dekódolás az OpenCV `FaceDetectorYN`-jét követi (a PicasaPy azt hívja):
//! 960 képpontos hosszabb oldal, BGR, 0-255, 32 többszörösére nullákkal kiegészítve. A PicasaPy
//! üres eredmény utáni második és harmadik menete, valamint az SFace-vektorok itt nincsenek meg.
//! A modell bemenetét a yunet_dinamikus.py teszi tetszőleges méretűvé.

use std::path::Path;
use std::time::{Duration, Instant};

use image::RgbImage;
use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;

use crate::belyegkep;

const HOSSZABB_OLDAL: u32 = 960;
const PONTSZAM_KUSZOB: f32 = 0.9;
const NMS_KUSZOB: f32 = 0.3;
/// Ha a modell nem tölthető be: a következtetés helyett ennyi pörgetés képenként (becslés).
const HELYETTESITO: Duration = Duration::from_millis(150);

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
    /// A modell nem töltődött be; a szöveg az ok.
    Helyettesito(String),
}

impl Detektor {
    pub fn betolt(modell: &Path) -> Self {
        match munkamenet(modell) {
            Ok(s) => Detektor::YuNet(s),
            Err(e) => Detektor::Helyettesito(e),
        }
    }

    pub fn helyettesito_oka(&self) -> Option<&str> {
        match self {
            Detektor::Helyettesito(ok) => Some(ok),
            Detektor::YuNet(_) => None,
        }
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
            Detektor::Helyettesito(_) => {
                porget(HELYETTESITO);
                Ok(0)
            }
        }
    }
}

fn szoveg<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Egy szálon fut (a hívó szálán), így annak alacsony prioritása érvényes rá.
fn munkamenet(modell: &Path) -> Result<Session, String> {
    if !modell.is_file() {
        return Err(format!("nincs modellfajl: {}", modell.display()));
    }
    Session::builder()
        .map_err(szoveg)?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(szoveg)?
        .with_intra_threads(1)
        .map_err(szoveg)?
        .commit_from_file(modell)
        .map_err(szoveg)
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

fn porget(ido: Duration) {
    let kezdet = Instant::now();
    let mut x = 1.0f64;
    while kezdet.elapsed() < ido {
        x = std::hint::black_box((x * 1.000_001).sin() + 1.0);
    }
    std::hint::black_box(x);
}

#[cfg(test)]
mod tesztek {
    use super::*;

    #[test]
    fn a_tensor_bgr_es_kiegeszitett() {
        let mut kep = RgbImage::new(40, 30);
        kep.put_pixel(1, 0, image::Rgb([10, 20, 30]));
        let b = tensor_adat(&kep);
        assert_eq!((b.szel, b.mag), (64, 32));
        let sik = 64 * 32;
        assert_eq!(
            [b.adat[1], b.adat[sik + 1], b.adat[2 * sik + 1]],
            [30.0, 20.0, 10.0]
        );
    }

    #[test]
    fn a_dekodolas_a_cella_helyere_teszi_a_keretet() {
        let cls = [0.0, 0.0, 0.0, 1.0];
        let obj = [1.0, 1.0, 1.0, 0.9];
        let bbox = [0.0; 16];
        let j = dekodol_szint(&cls, &obj, &bbox, 2, 8);
        assert_eq!(j.len(), 1);
        // a 3. cella (1. oszlop, 1. sor): középpont 8;8, oldal 8
        assert_eq!((j[0].x, j[0].y, j[0].szel, j[0].mag), (4.0, 4.0, 8.0, 8.0));
    }

    #[test]
    fn az_atfedo_keretekbol_a_legerosebb_marad() {
        let k = |x, pontszam| Jelolt {
            x,
            y: 0.0,
            szel: 10.0,
            mag: 10.0,
            pontszam,
        };
        let marad = nms(vec![k(1.0, 0.91), k(0.0, 0.99), k(50.0, 0.95)], 0.3);
        assert_eq!(marad.iter().map(|j| j.x).collect::<Vec<_>>(), [0.0, 50.0]);
    }

    #[test]
    fn modell_nelkul_helyettesito_munka() {
        let d = Detektor::betolt(Path::new("nincs-ilyen-modell.onnx"));
        assert!(d.helyettesito_oka().is_some());
    }
}
