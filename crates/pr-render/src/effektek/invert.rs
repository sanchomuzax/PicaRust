pub const NEV: &str = "invert";
pub const LANC_NEV: &str = "Invert";
pub const PARAMETER_COUNT: usize = 0;

pub fn alkalmaz(
    kep: &image::RgbImage,
    parameterek: &[String],
) -> Result<image::RgbImage, crate::Hiba> {
    if parameterek.iter().any(|ertek| !ertek.is_empty()) {
        return Err(crate::Hiba::HibasParameter {
            effekt: NEV.to_owned(),
            ok: "nem fogad paramétert".to_owned(),
        });
    }

    let mut eredmeny = kep.clone();
    for pixel in eredmeny.pixels_mut() {
        for csatorna in &mut pixel.0 {
            *csatorna = u8::MAX - *csatorna;
        }
    }
    Ok(eredmeny)
}
