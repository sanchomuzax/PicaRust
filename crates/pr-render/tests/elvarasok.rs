use image::{ColorType, DynamicImage, GenericImageView, RgbImage};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn munkaterulet_gyokere() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../")
}

fn jegyzek() -> Result<Value, String> {
    let ut = munkaterulet_gyokere().join("elvarasok/JEGYZEK.json");
    let szoveg = fs::read_to_string(&ut)
        .map_err(|hiba| format!("Nem olvasható az elvárásjegyzék {}: {hiba}", ut.display()))?;
    serde_json::from_str(&szoveg)
        .map_err(|hiba| format!("Hibás JSON az elvárásjegyzékben {}: {hiba}", ut.display()))
}

fn esetek(adatok: &Value) -> Option<&Vec<Value>> {
    adatok.get("effekt_esetek").and_then(Value::as_array)
}

fn osszesitest_kiir(futott: usize, kimaradt: usize) -> Result<(), String> {
    let uzenet = format!("Effekt elvárások: futott={futott}, kimaradt={kimaradt}\n");
    std::io::stdout()
        .lock()
        .write_all(uzenet.as_bytes())
        .map_err(|hiba| format!("Nem írható ki az effekt-összesítés: {hiba}"))
}

fn ertekes_szoveg<'a>(eset: &'a Value, kulcs: &str) -> Option<&'a str> {
    eset.get(kulcs)
        .and_then(Value::as_str)
        .filter(|szoveg| !szoveg.trim().is_empty())
}

fn meret(eset: &Value) -> Option<(u32, u32)> {
    let ertekek = eset.get("meret")?.as_array()?;
    if ertekek.len() != 2 {
        return None;
    }
    let szelesseg = u32::try_from(ertekek.first()?.as_u64()?).ok()?;
    let magassag = u32::try_from(ertekek.get(1)?.as_u64()?).ok()?;
    Some((szelesseg, magassag))
}

fn lanc_ellenorzese(lanc: &str, vart_effekt: &str) -> Result<pr_render::lanc::Ertelmezett, String> {
    let ertelmezett = pr_render::lanc::ertelmez(lanc);
    if let Some(ok) = &ertelmezett.elvagva {
        return Err(format!("az effektlánc elvágódott: {ok}"));
    }
    if ertelmezett.lepesek.len() != 1 {
        return Err(format!(
            "egy effektlépés helyett {} lépés értelmeződött",
            ertelmezett.lepesek.len()
        ));
    }
    if ertelmezett.lepesek[0].nev != vart_effekt {
        return Err(format!(
            "a lánc effektje {vart_effekt} helyett {}",
            ertelmezett.lepesek[0].nev
        ));
    }
    Ok(ertelmezett)
}

fn kep_betoltese(gyoker: &Path, nev: &str, vart_meret: (u32, u32)) -> Result<RgbImage, String> {
    let ut = gyoker.join(nev);
    let kep =
        image::open(&ut).map_err(|hiba| format!("Nem olvasható a kép {}: {hiba}", ut.display()))?;
    if kep.color() != ColorType::Rgb8 {
        return Err(format!(
            "A kép nem 8 bites RGB: {} ({:?})",
            ut.display(),
            kep.color()
        ));
    }
    if kep.dimensions() != vart_meret {
        return Err(format!(
            "A kép mérete nem egyezik: {} ({:?}, elvárt {:?})",
            ut.display(),
            kep.dimensions(),
            vart_meret
        ));
    }
    match kep {
        DynamicImage::ImageRgb8(rgb) => Ok(rgb),
        _ => Err(format!("A kép nem 8 bites RGB: {}", ut.display())),
    }
}

fn elteres(eredmeny: &RgbImage, elvart: &RgbImage) -> (u64, u8) {
    if eredmeny.dimensions() != elvart.dimensions() {
        return (
            u64::from(elvart.width()) * u64::from(elvart.height()),
            u8::MAX,
        );
    }
    let mut hibas = 0;
    let mut legnagyobb = 0;
    for (kapott, vart) in eredmeny.pixels().zip(elvart.pixels()) {
        let mut pixel_eltert = false;
        for csatorna in 0..3 {
            let kulonbseg = kapott[csatorna].abs_diff(vart[csatorna]);
            pixel_eltert |= kulonbseg > 0;
            legnagyobb = legnagyobb.max(kulonbseg);
        }
        if pixel_eltert {
            hibas += 1;
        }
    }
    (hibas, legnagyobb)
}

fn mezo_hiba(eset: &Value, kulcs: &str, index: usize, azon: &str) -> Option<String> {
    ertekes_szoveg(eset, kulcs).map_or_else(
        || {
            Some(format!(
                "{azon}: hiányzó, üres vagy hibás típusú {kulcs} mező (eset #{index})"
            ))
        },
        |_| None,
    )
}

#[test]
fn ismert_effektek_mind_szerepelnek_a_jegyzekben() {
    let jegyzek = jegyzek();
    assert!(
        jegyzek.is_ok(),
        "Elvárásjegyzék hiba: {:?}",
        jegyzek.as_ref().err()
    );
    let adatok = match jegyzek {
        Ok(adatok) => adatok,
        Err(_) => Value::Null,
    };
    let esetek = esetek(&adatok);
    assert!(
        esetek.is_some(),
        "Hiányzik vagy hibás az effekt_esetek lista"
    );

    let mut hibak = Vec::new();
    let mut nevek = HashSet::new();
    for (index, eset) in esetek.into_iter().flatten().enumerate() {
        let azon = ertekes_szoveg(eset, "azonosito")
            .map_or_else(|| format!("eset #{index}"), str::to_owned);
        if let Some(hiba) = mezo_hiba(eset, "effekt", index, &azon) {
            hibak.push(hiba);
        } else if let Some(nev) = ertekes_szoveg(eset, "effekt") {
            nevek.insert(nev.to_owned());
        }
    }
    for nev in pr_render::ismert_effektek() {
        if !nevek.contains(nev) {
            hibak.push(format!("Az effekt nincs az elvárásjegyzékben: {nev}"));
        }
    }
    assert!(hibak.is_empty(), "{}", hibak.join("\n"));
}

#[test]
fn ismert_effektek_elvarasai_keppontra_egyeznek() {
    let jegyzek = jegyzek();
    if let Err(hiba) = &jegyzek {
        let kiiras = osszesitest_kiir(0, 0);
        assert!(kiiras.is_ok(), "{:?}", kiiras.err());
        assert!(hiba.is_empty(), "{hiba}");
    }
    let adatok = match jegyzek {
        Ok(adatok) => adatok,
        Err(_) => Value::Null,
    };
    let gyoker = munkaterulet_gyokere();
    let ismertek: HashSet<String> = pr_render::ismert_effektek()
        .into_iter()
        .map(str::to_owned)
        .collect();
    let mut futott = 0;
    let mut kimaradt = 0;
    let mut hibak = Vec::new();
    let lista: &[Value] = match esetek(&adatok) {
        Some(lista) => lista,
        None => {
            hibak.push("Hiányzik vagy hibás az effekt_esetek lista".to_owned());
            &[]
        }
    };

    for (index, eset) in lista.iter().enumerate() {
        let azon = ertekes_szoveg(eset, "azonosito")
            .map_or_else(|| format!("eset #{index}"), str::to_owned);
        let mut mezohiba = false;
        for kulcs in ["azonosito", "effekt", "lanc", "bemenet", "ut"] {
            if let Some(hiba) = mezo_hiba(eset, kulcs, index, &azon) {
                hibak.push(hiba);
                mezohiba = true;
            }
        }
        if mezohiba {
            continue;
        }

        let Some(effekt) = ertekes_szoveg(eset, "effekt") else {
            continue;
        };
        let Some(lanc) = ertekes_szoveg(eset, "lanc") else {
            continue;
        };
        let Some(bemenet_ut) = ertekes_szoveg(eset, "bemenet") else {
            continue;
        };
        let Some(elvart_ut) = ertekes_szoveg(eset, "ut") else {
            continue;
        };
        if !ismertek.contains(effekt) {
            kimaradt += 1;
            continue;
        }

        let Some(vart_meret) = meret(eset) else {
            hibak.push(format!("{azon}: hiányzó vagy hibás típusú meret mező"));
            continue;
        };
        if let Err(hiba) = lanc_ellenorzese(lanc, effekt) {
            hibak.push(format!("{azon}: {hiba}"));
            continue;
        }

        let bemenet = kep_betoltese(&gyoker, bemenet_ut, vart_meret);
        let elvart = kep_betoltese(&gyoker, elvart_ut, vart_meret);
        let (bemenet, elvart) = match (bemenet, elvart) {
            (Ok(bemenet), Ok(elvart)) => (bemenet, elvart),
            (Err(hiba), Ok(_)) => {
                hibak.push(format!("{azon}: {hiba}"));
                continue;
            }
            (Ok(_), Err(hiba)) => {
                hibak.push(format!("{azon}: {hiba}"));
                continue;
            }
            (Err(bemenet_hiba), Err(elvart_hiba)) => {
                hibak.push(format!("{azon}: {bemenet_hiba}"));
                hibak.push(format!("{azon}: {elvart_hiba}"));
                continue;
            }
        };
        futott += 1;

        match pr_render::alkalmaz_szigoruan(lanc, &bemenet) {
            Ok(eredmeny) => {
                let (hibas_pixelek, legnagyobb_elteres) = elteres(&eredmeny, &elvart);
                if hibas_pixelek > 0 {
                    hibak.push(format!(
                        "{azon}: eltérő képpontok: {hibas_pixelek}, legnagyobb csatornánkénti eltérés: {legnagyobb_elteres}"
                    ));
                }
            }
            Err(hiba) => hibak.push(format!("{azon}: az effektlánc hibát adott: {hiba}")),
        }
    }

    let kiiras = osszesitest_kiir(futott, kimaradt);
    if let Err(hiba) = kiiras {
        hibak.push(hiba);
    }
    assert!(hibak.is_empty(), "{}", hibak.join("\n"));
    assert!(futott > 0, "Egy ismert effekt esete sem futott");
}

#[test]
fn valtozatlan_kimenetnel_is_elutasitja_a_rossz_effektnevet() {
    let bemenet = RgbImage::from_pixel(1, 1, image::Rgb([80, 120, 190]));
    let elvart = bemenet.clone();

    assert_eq!(elteres(&bemenet, &elvart), (0, 0));
    assert_eq!(
        pr_render::alkalmaz_szigoruan("Boost=1,0;", &bemenet),
        Ok(elvart)
    );
    let ellenorzes = lanc_ellenorzese("Invert=1;", "boost");
    assert!(
        ellenorzes.is_err(),
        "A változatlan kimenet nem fedheti el az eltérő effektnevet"
    );
}
