use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use pr_index::{Hiba, Index};
use pr_scan::{Foto, Mappa};

static KOVETKEZO: AtomicU64 = AtomicU64::new(0);

struct TesztMappa(PathBuf);

impl TesztMappa {
    fn uj(nev: &str) -> Result<Self, String> {
        let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
        let ido = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|hiba| hiba.to_string())?
            .as_nanos();
        let sorszam = KOVETKEZO.fetch_add(1, Ordering::Relaxed);
        let ut = munkafa.join(".tmp").join(format!(
            "pr-index-{nev}-{}-{ido}-{sorszam}",
            std::process::id()
        ));
        fs::create_dir_all(&ut).map_err(|hiba| hiba.to_string())?;
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
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn mappa(ut: &Path, nev: &str, kepek: &[(&str, u64, u128)]) -> Mappa {
    Mappa {
        ut: ut.to_path_buf(),
        nev: nev.to_string(),
        fotok: kepek
            .iter()
            .map(|(nev, meret, mtime_ns)| Foto {
                ut: ut.join(nev),
                meret: *meret,
                mtime_ns: *mtime_ns,
            })
            .collect(),
    }
}

fn regi_ut(fajl: &Path) -> PathBuf {
    let mut regi = fajl.as_os_str().to_os_string();
    regi.push(".regi");
    PathBuf::from(regi)
}

fn ut_suffixel(ut: &Path, kiegeszito: &str) -> PathBuf {
    let mut nev = ut.as_os_str().to_os_string();
    nev.push(kiegeszito);
    PathBuf::from(nev)
}

#[test]
fn hianyzo_adatbazis_fajlt_letrehozza() -> Result<(), String> {
    let gy = TesztMappa::uj("letrehozas")?;
    let fajl = gy.join("index.sqlite");
    let _index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    assert!(fajl.is_file());
    Ok(())
}

#[test]
fn serult_adatbazist_felretesz_es_ures_indexet_nyit() -> Result<(), String> {
    let gy = TesztMappa::uj("serult")?;
    let fajl = gy.join("index.sqlite");
    let eredeti = b"veletlen serult adat";
    fs::write(&fajl, eredeti).map_err(|hiba| hiba.to_string())?;
    let regi = regi_ut(&fajl);
    fs::write(&regi, b"korabbi index").map_err(|hiba| hiba.to_string())?;

    let index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;

    assert!(fajl.is_file());
    assert_eq!(
        fs::read(&regi).map_err(|hiba| hiba.to_string())?,
        b"korabbi index"
    );
    let masodik_regi = PathBuf::from(format!("{}.1", regi.display()));
    assert_eq!(
        fs::read(masodik_regi).map_err(|hiba| hiba.to_string())?,
        eredeti
    );
    assert!(index.mappak().map_err(|hiba| hiba.to_string())?.is_empty());
    Ok(())
}

#[test]
fn zarolt_adatbazist_nem_tesz_felre() -> Result<(), String> {
    let gy = TesztMappa::uj("zarolas")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let mut kezdeti = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    kezdeti
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 12, 34)])])
        .map_err(|hiba| hiba.to_string())?;
    drop(kezdeti);
    let zar = rusqlite::Connection::open(&fajl).map_err(|hiba| hiba.to_string())?;
    zar.execute_batch("BEGIN EXCLUSIVE; UPDATE mappak SET nev = 'album-masik'")
        .map_err(|hiba| hiba.to_string())?;

    match Index::megnyit(&fajl) {
        Err(Hiba::Megnyitas(_)) => {}
        Err(hiba) => return Err(format!("nem megnyitási hibát adott: {hiba}")),
        Ok(mut index) => {
            let muvelet = index.frissit(&[]);
            assert!(matches!(muvelet, Err(Hiba::Lekerdezes(_))));
        }
    }

    assert!(fajl.is_file());
    assert!(!regi_ut(&fajl).exists());
    zar.execute_batch("ROLLBACK")
        .map_err(|hiba| hiba.to_string())?;
    Ok(())
}

#[test]
fn serult_adatbazis_mellekallomanyait_is_felreteszi() -> Result<(), String> {
    let gy = TesztMappa::uj("naplo")?;
    let fajl = gy.join("index.sqlite");
    fs::write(&fajl, b"serult fo fajl").map_err(|hiba| hiba.to_string())?;
    let kiserok = ["-journal", "-wal", "-shm"];
    for kiegeszito in kiserok {
        fs::write(ut_suffixel(&fajl, kiegeszito), kiegeszito.as_bytes())
            .map_err(|hiba| hiba.to_string())?;
    }

    drop(Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?);

    for kiegeszito in kiserok {
        assert!(!ut_suffixel(&fajl, kiegeszito).exists());
        let regi_kisero = ut_suffixel(&regi_ut(&fajl), kiegeszito);
        if regi_kisero.exists() {
            assert_eq!(
                fs::read(regi_kisero).map_err(|hiba| hiba.to_string())?,
                kiegeszito.as_bytes()
            );
        }
    }
    Ok(())
}

#[test]
fn ismeretlen_schemaverziot_felretesz_es_ures_indexet_nyit() -> Result<(), String> {
    let gy = TesztMappa::uj("ismeretlen-verzio")?;
    let fajl = gy.join("index.sqlite");
    drop(Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?);
    let kapcsolat = rusqlite::Connection::open(&fajl).map_err(|hiba| hiba.to_string())?;
    kapcsolat
        .pragma_update(None, "user_version", 99_i64)
        .map_err(|hiba| hiba.to_string())?;
    drop(kapcsolat);

    let index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;

    assert!(fajl.is_file());
    assert!(regi_ut(&fajl).is_file());
    assert!(index.mappak().map_err(|hiba| hiba.to_string())?.is_empty());
    Ok(())
}

#[test]
fn ismetelt_frissites_nem_duplaz() -> Result<(), String> {
    let gy = TesztMappa::uj("ismetles")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let bemenet = [mappa(&album, "album", &[("foto.jpg", 12, 34)])];
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;

    let elso = index.frissit(&bemenet).map_err(|hiba| hiba.to_string())?;
    let masodik = index.frissit(&bemenet).map_err(|hiba| hiba.to_string())?;

    assert_eq!(elso.uj, 1);
    assert_eq!((elso.mappak, elso.kepek), (1, 1));
    assert_eq!((masodik.uj, masodik.modosult, masodik.torolt), (0, 0, 0));
    assert_eq!(
        index.kepek(&album).map_err(|hiba| hiba.to_string())?.len(),
        1
    );
    Ok(())
}

#[test]
fn frissites_torli_a_bejarasbol_kimaradt_kepeket_es_mappakat() -> Result<(), String> {
    let gy = TesztMappa::uj("torles")?;
    let fajl = gy.join("index.sqlite");
    let megmarad = gy.join("megmarad");
    let torlendo = gy.join("torlendo");
    let kezdeti = [
        mappa(&megmarad, "megmarad", &[("marad.jpg", 1, 1)]),
        mappa(&torlendo, "torlendo", &[("torol.jpg", 2, 2)]),
    ];
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index.frissit(&kezdeti).map_err(|hiba| hiba.to_string())?;

    let osszesites = index
        .frissit(&[kezdeti[0].clone()])
        .map_err(|hiba| hiba.to_string())?;

    assert_eq!(osszesites.torolt, 1);
    assert_eq!(index.mappak().map_err(|hiba| hiba.to_string())?.len(), 1);
    assert!(index
        .kepek(&torlendo)
        .map_err(|hiba| hiba.to_string())?
        .is_empty());
    Ok(())
}

#[test]
fn kep_torlesekor_a_mappa_megmarad() -> Result<(), String> {
    let gy = TesztMappa::uj("kep-torles")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 12, 34)])])
        .map_err(|hiba| hiba.to_string())?;

    let osszesites = index
        .frissit(&[mappa(&album, "album", &[])])
        .map_err(|hiba| hiba.to_string())?;

    assert_eq!(
        (osszesites.uj, osszesites.modosult, osszesites.torolt),
        (0, 0, 1)
    );
    assert_eq!(osszesites.mappak, 1);
    assert_eq!(osszesites.kepek, 0);
    assert_eq!(index.mappak().map_err(|hiba| hiba.to_string())?.len(), 1);
    assert!(index
        .kepek(&album)
        .map_err(|hiba| hiba.to_string())?
        .is_empty());
    Ok(())
}

#[test]
fn modosult_meretu_vagy_idobelyegu_kepet_szamol() -> Result<(), String> {
    let gy = TesztMappa::uj("modositas")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 12, 34)])])
        .map_err(|hiba| hiba.to_string())?;

    let osszesites = index
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 13, 34)])])
        .map_err(|hiba| hiba.to_string())?;

    assert_eq!(osszesites.modosult, 1);
    let csak_ido = index
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 13, 35)])])
        .map_err(|hiba| hiba.to_string())?;
    assert_eq!((csak_ido.uj, csak_ido.modosult, csak_ido.torolt), (0, 1, 0));
    Ok(())
}

#[test]
fn athelyezett_kepet_toroltnek_es_ujnak_szamol() -> Result<(), String> {
    let gy = TesztMappa::uj("athelyezes")?;
    let fajl = gy.join("index.sqlite");
    let regi_album = gy.join("regi");
    let uj_album = gy.join("uj");
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[mappa(&regi_album, "regi", &[("foto.jpg", 12, 34)])])
        .map_err(|hiba| hiba.to_string())?;

    let osszesites = index
        .frissit(&[mappa(&uj_album, "uj", &[("foto.jpg", 12, 34)])])
        .map_err(|hiba| hiba.to_string())?;

    assert_eq!(
        (osszesites.uj, osszesites.modosult, osszesites.torolt),
        (1, 0, 1)
    );
    Ok(())
}

#[test]
fn mappak_es_kepek_utvonal_szerint_rendezettek() -> Result<(), String> {
    let gy = TesztMappa::uj("rendezes")?;
    let fajl = gy.join("index.sqlite");
    let a = gy.join("a");
    let z = gy.join("z");
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[
            mappa(&z, "z", &[("z.jpg", 1, 1), ("a.jpg", 1, 1)]),
            mappa(&a, "a", &[("foto.jpg", 1, 1)]),
        ])
        .map_err(|hiba| hiba.to_string())?;

    let mappak = index.mappak().map_err(|hiba| hiba.to_string())?;
    let kepek = index.kepek(&z).map_err(|hiba| hiba.to_string())?;

    assert!(mappak[0].ut < mappak[1].ut);
    assert!(kepek[0].ut < kepek[1].ut);
    Ok(())
}

#[test]
fn keres_ekezet_es_betu_nagysag_nelkul_es_szo_szerint() -> Result<(), String> {
    let gy = TesztMappa::uj("kereses")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[mappa(
            &album,
            "album",
            &[
                ("Árvíz_%_TŰRŐ.jpg", 1, 1),
                ("arviz-xx-turo.jpg", 1, 1),
                ("ÁÉÍÓÖŐÚÜŰ.jpg", 1, 1),
                ("aeiooouuu.jpg", 1, 1),
                ("Straße.jpg", 1, 1),
                ("strasse.jpg", 1, 1),
                ("Ångström.jpg", 1, 1),
                ("angstrom.jpg", 1, 1),
            ],
        )])
        .map_err(|hiba| hiba.to_string())?;

    let egyezes = index.keres("ARVIZ_%_").map_err(|hiba| hiba.to_string())?;
    let teljes_magyar_keszlet = index.keres("aeiooouuu").map_err(|hiba| hiba.to_string())?;
    let ekezetes_kereses = index.keres("áéíóöőúüű").map_err(|hiba| hiba.to_string())?;
    let nemet_kereses = index.keres("strasse").map_err(|hiba| hiba.to_string())?;
    let nem_magyar_ekezet = index.keres("angstrom").map_err(|hiba| hiba.to_string())?;
    let ures = index.keres("   ").map_err(|hiba| hiba.to_string())?;

    assert_eq!(egyezes.len(), 1);
    assert!(egyezes[0].ut.ends_with("Árvíz_%_TŰRŐ.jpg"));
    assert_eq!(teljes_magyar_keszlet.len(), 2);
    assert!(teljes_magyar_keszlet
        .iter()
        .any(|kep| kep.ut.ends_with("ÁÉÍÓÖŐÚÜŰ.jpg")));
    assert!(teljes_magyar_keszlet
        .iter()
        .any(|kep| kep.ut.ends_with("aeiooouuu.jpg")));
    assert_eq!(ekezetes_kereses.len(), 2);
    assert!(ekezetes_kereses
        .iter()
        .any(|kep| kep.ut.ends_with("aeiooouuu.jpg")));
    assert_eq!(nemet_kereses.len(), 1);
    assert!(nemet_kereses[0].ut.ends_with("strasse.jpg"));
    assert_eq!(nem_magyar_ekezet.len(), 1);
    assert!(nem_magyar_ekezet[0].ut.ends_with("angstrom.jpg"));
    assert!(ures.is_empty());
    Ok(())
}

#[test]
fn mtime_ns_u128_veszteseg_nelkul_visszajon() -> Result<(), String> {
    let gy = TesztMappa::uj("mtime")?;
    let fajl = gy.join("index.sqlite");
    let album = gy.join("album");
    let mtime_ns = u128::MAX;
    let mut index = Index::megnyit(&fajl).map_err(|hiba| hiba.to_string())?;
    index
        .frissit(&[mappa(&album, "album", &[("foto.jpg", 1, mtime_ns)])])
        .map_err(|hiba| hiba.to_string())?;

    let kepek = index.kepek(&album).map_err(|hiba| hiba.to_string())?;

    assert_eq!(kepek[0].mtime_ns, mtime_ns);
    Ok(())
}
