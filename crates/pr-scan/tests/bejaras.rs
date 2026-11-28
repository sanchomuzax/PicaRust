use std::ops::Deref;
use std::path::{Path, PathBuf};

use pr_scan::{almappak, fa_beolvas, kep_e, Foto, Mappa};

struct TesztMappa(PathBuf);

impl TesztMappa {
    fn uj(nev: &str) -> Result<Self, String> {
        let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
        let ut = munkafa
            .join(".tmp")
            .join(format!("picarust-{nev}-{}", std::process::id()));
        if ut.exists() {
            std::fs::remove_dir_all(&ut).map_err(|e| e.to_string())?;
        }
        std::fs::create_dir_all(&ut).map_err(|e| e.to_string())?;
        Ok(Self(ut))
    }
}

impl Deref for TesztMappa {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TesztMappa {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn csak_a_kepet_tartalmazo_mappak_jonnek() -> Result<(), String> {
    let gy = TesztMappa::uj("fa")?;
    std::fs::create_dir_all(gy.join("b")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(gy.join("a")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(gy.join("ures")).map_err(|e| e.to_string())?;
    std::fs::write(gy.join("a/2.PNG"), b"x").map_err(|e| e.to_string())?;
    std::fs::write(gy.join("a/1.jpg"), b"x").map_err(|e| e.to_string())?;
    std::fs::write(gy.join("b/3.jpeg"), b"x").map_err(|e| e.to_string())?;
    std::fs::write(gy.join("b/nem.txt"), b"x").map_err(|e| e.to_string())?;
    let m: Vec<Mappa> = fa_beolvas(&gy);
    assert_eq!(
        m.iter().map(|m| m.nev.as_str()).collect::<Vec<_>>(),
        ["a", "b"]
    );
    assert_eq!(m[0].fotok.len(), 2);
    assert!(m[0].fotok[0].ut.ends_with("1.jpg"));
    assert_eq!(m[1].fotok.len(), 1);
    assert_eq!(almappak(&gy).len(), 3);
    Ok(())
}

#[test]
fn ures_es_nem_letezo_keptar_ures() -> Result<(), String> {
    let gy = TesztMappa::uj("ures")?;
    assert!(fa_beolvas(&gy).is_empty());
    std::fs::remove_dir_all(&*gy).map_err(|e| e.to_string())?;
    assert!(fa_beolvas(&gy).is_empty());
    Ok(())
}

#[test]
fn ekezetes_szokoztartalmu_keptar_husz_kepet_beolvas_es_nem_ir_vissza() -> Result<(), String> {
    let gy = TesztMappa::uj("ekezetes")?;
    let album = gy.join("Képek próba").join("árvíztűrő");
    std::fs::create_dir_all(&album).map_err(|e| e.to_string())?;
    for index in 0..20 {
        std::fs::write(album.join(format!("foto-{index}.jpg")), b"").map_err(|e| e.to_string())?;
    }
    let before = std::fs::read_dir(&album)
        .map_err(|e| e.to_string())?
        .map(|entry| {
            entry
                .map_err(|e| e.to_string())
                .map(|entry| entry.file_name())
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mappak: Vec<Mappa> = fa_beolvas(&gy);
    let after = std::fs::read_dir(&album)
        .map_err(|e| e.to_string())?
        .map(|entry| {
            entry
                .map_err(|e| e.to_string())
                .map(|entry| entry.file_name())
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(mappak.len(), 1);
    assert_eq!(mappak[0].nev, "árvíztűrő");
    assert_eq!(mappak[0].fotok.len(), 20);
    assert_eq!(before.len(), after.len());
    assert!(before.iter().all(|name| after.contains(name)));
    Ok(())
}

#[test]
fn kep_e_a_jpg_jpeg_es_png_kiterjesztest_az_esetbetutol_fuggetlenul_fogadja() {
    for ut in ["foto.jpg", "foto.JPEG", "foto.PnG"] {
        assert!(kep_e(Path::new(ut)), "nem kepkent felismerve: {ut}");
    }
    for ut in ["foto.gif", "foto.png.txt", "foto"] {
        assert!(!kep_e(Path::new(ut)), "kepkent felismerve: {ut}");
    }
}

#[test]
fn a_foto_es_mappa_mezoi_nyilvanosak_es_valtozatlan_tipusuak() {
    let foto = Foto {
        ut: PathBuf::from("album/foto.jpg"),
        meret: 42,
        mtime_ns: 123,
    };
    let mappa = Mappa {
        ut: PathBuf::from("album"),
        nev: "album".to_string(),
        fotok: vec![foto.clone()],
    };

    assert_eq!(mappa.fotok, [foto]);
}
