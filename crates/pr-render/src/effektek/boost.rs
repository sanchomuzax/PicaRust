pub const NEV: &str = "boost";
pub const LANC_NEV: &str = "Boost";
pub const PARAMETER_COUNT: usize = 1;

const KONTRASZT_TABLA: [f64; 101] = [
    0.00, 0.01, 0.02, 0.04, 0.05, 0.06, 0.07, 0.08, 0.10, 0.11, 0.12, 0.14, 0.15, 0.16,
    0.17, 0.18, 0.20, 0.21, 0.22, 0.24, 0.25, 0.27, 0.28, 0.30, 0.32, 0.34, 0.36, 0.38,
    0.40, 0.42, 0.44, 0.46, 0.48, 0.50, 0.53, 0.56, 0.59, 0.62, 0.65, 0.68, 0.71, 0.74,
    0.77, 0.80, 0.83, 0.86, 0.89, 0.92, 0.95, 0.98, 1.00, 1.06, 1.12, 1.18, 1.24, 1.30,
    1.36, 1.42, 1.48, 1.54, 1.60, 1.66, 1.72, 1.78, 1.84, 1.90, 1.96, 2.00, 2.12, 2.25,
    2.37, 2.50, 2.62, 2.75, 2.87, 3.00, 3.20, 3.40, 3.60, 3.80, 4.00, 4.30, 4.70, 4.90,
    5.00, 5.50, 6.00, 6.50, 6.80, 7.00, 7.30, 7.50, 7.80, 8.00, 8.40, 8.70, 9.00, 9.40,
    9.60, 9.80, 10.00,
];

pub fn alkalmaz(
    kep: &image::RgbImage,
    parameterek: &[String],
) -> Result<image::RgbImage, crate::Hiba> {
    let impact = hatas(parameterek)?;
    let fenyero = (impact * -20.0 / 50.0).clamp(-100.0, 100.0);
    let telitettseg = impact * 20.0 / 50.0;
    let kontraszt = (impact * 40.0 / 50.0).clamp(-100.0, 100.0);

    let saturation = telitettsegi_matrix(telitettseg);
    let mut matrix = homogen(&saturation);
    if kontraszt != 0.0 || fenyero != 0.0 {
        let contrast = kontraszt_matrix(kontraszt);
        if let Some(contrast) = contrast {
            matrix = szorzas_x87(matrix, contrast);
        }
        matrix = szorzas_x87(matrix, fenyero_matrix(fenyero));
    }

    Ok(alkalmaz_matrixot(kep, &matrix))
}

fn hatas(parameterek: &[String]) -> Result<f64, crate::Hiba> {
    let Some(nyers) = parameterek.first() else {
        return Ok(50.0);
    };
    match nyers.trim().parse::<f64>() {
        Ok(ertek) if ertek.is_finite() => Ok(ertek.clamp(0.0, 100.0)),
        Ok(_) => Ok(50.0),
        Err(_) => Err(hibas_parameter(nyers.clone())),
    }
}

fn hibas_parameter(ok: String) -> crate::Hiba {
    crate::Hiba::HibasParameter {
        effekt: NEV.to_owned(),
        ok,
    }
}

fn telitettsegi_matrix(telitettseg: f64) -> [[f32; 3]; 3] {
    let erosites = if telitettseg > 0.0 {
        1.0 + (telitettseg * 3.0) / 100.0
    } else {
        1.0 + telitettseg / 100.0
    };
    let w = 1.0 - erosites;
    let voros = w * 0.3086;
    let zold = w * 0.6094;
    let kek = w * 0.0820;
    [
        [(erosites + voros) as f32, zold as f32, kek as f32],
        [voros as f32, (erosites + zold) as f32, kek as f32],
        [voros as f32, zold as f32, (erosites + kek) as f32],
    ]
}

fn kontraszt_gorbe(kontraszt: f64) -> f64 {
    if kontraszt == 0.0 {
        return 0.0;
    }
    if kontraszt < 0.0 {
        return kontraszt / 100.0;
    }
    if kontraszt >= 100.0 {
        return KONTRASZT_TABLA[100];
    }
    let index = kontraszt as usize;
    let tort = kontraszt - index as f64;
    if tort < 1e-9 {
        KONTRASZT_TABLA[index]
    } else {
        (1.0 - tort) * KONTRASZT_TABLA[index] + tort * KONTRASZT_TABLA[index + 1]
    }
}

fn homogen(matrix: &[[f32; 3]; 3]) -> [[f32; 4]; 4] {
    let mut eredmeny = azonossag();
    for sor in 0..3 {
        for oszlop in 0..3 {
            eredmeny[sor][oszlop] = matrix[sor][oszlop];
        }
    }
    eredmeny
}

fn kontraszt_matrix(kontraszt: f64) -> Option<[[f32; 4]; 4]> {
    let k = 1.0 + kontraszt_gorbe(kontraszt.clamp(-100.0, 100.0));
    if (k - 1.0).abs() < 1e-6 {
        return None;
    }
    let mut eredmeny = azonossag();
    for (sor, elem) in eredmeny.iter_mut().enumerate().take(3) {
        elem[sor] = k as f32;
        elem[3] = ((1.0 - k) * 127.0 * 0.5) as f32;
    }
    Some(eredmeny)
}

fn fenyero_matrix(fenyero: f64) -> [[f32; 4]; 4] {
    let mut eredmeny = azonossag();
    for elem in eredmeny.iter_mut().take(3) {
        elem[3] = fenyero.clamp(-100.0, 100.0) as f32;
    }
    eredmeny
}

fn azonossag() -> [[f32; 4]; 4] {
    let mut eredmeny = [[0.0; 4]; 4];
    for (index, sor) in eredmeny.iter_mut().enumerate() {
        sor[index] = 1.0;
    }
    eredmeny
}

fn szorzas_x87(elso: [[f32; 4]; 4], masodik: [[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut eredmeny = [[0.0; 4]; 4];
    for sor in 0..4 {
        for oszlop in 0..4 {
            let mut osszeg = 0.0_f32;
            for index in 0..4 {
                osszeg = (f64::from(osszeg)
                    + f64::from(elso[sor][index]) * f64::from(masodik[index][oszlop]))
                    as f32;
            }
            eredmeny[sor][oszlop] = osszeg;
        }
    }
    eredmeny
}

fn alkalmaz_matrixot(kep: &image::RgbImage, matrix: &[[f32; 4]; 4]) -> image::RgbImage {
    let mut eredmeny = kep.clone();
    let mut coeff = [[0_i64; 3]; 3];
    let mut bias = [0_i64; 3];
    for sor in 0..3 {
        for oszlop in 0..3 {
            let scaled = f64::from(matrix[sor][oszlop]) * 2048.0;
            coeff[sor][oszlop] = (scaled + if scaled < 0.0 { -0.5 } else { 0.5 }).trunc() as i64;
        }
        let scaled = f64::from(matrix[sor][3]) * 4.0;
        bias[sor] = (scaled + if scaled < 0.0 { -0.5 } else { 0.5 }).trunc() as i64 + 2;
    }

    for pixel in eredmeny.pixels_mut() {
        let bemenet = pixel.0;
        let mut kimenet = [0_u8; 3];
        for sor in 0..3 {
            let mut osszeg = bias[sor];
            for oszlop in 0..3 {
                let tag = coeff[sor][oszlop] * i64::from(bemenet[oszlop]);
                osszeg += tag >> 9;
            }
            kimenet[sor] = (osszeg >> 2).clamp(0, 255) as u8;
        }
        *pixel = image::Rgb(kimenet);
    }
    eredmeny
}
