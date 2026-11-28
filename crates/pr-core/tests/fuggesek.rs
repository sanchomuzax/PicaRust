//! A csomagok közti belső (`path`) függések csak a gyökér `fuggesek.toml`
//! listájából valók. A teszt a `Cargo.toml` fájlokat egyszerű szövegelemzéssel
//! olvassa, külső csomag nélkül.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn gyoker() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "a pr-core nem a munkaterület alatt van".to_string())
}

fn olvas(ut: &Path) -> Result<String, String> {
    fs::read_to_string(ut).map_err(|hiba| format!("{} nem olvasható: {hiba}", ut.display()))
}

/// Az `engedett = [ "a -> b", ... ]` lista elemei.
fn engedett_elek(szoveg: &str) -> Result<BTreeSet<String>, String> {
    let mut elek = BTreeSet::new();
    let mut listaban = false;
    let mut talalt = false;
    for sor in szoveg.lines() {
        let sor = sor.split('#').next().unwrap_or("").trim();
        if sor.starts_with("engedett") && sor.contains('=') {
            listaban = true;
            talalt = true;
        } else if listaban && sor.starts_with(']') {
            listaban = false;
        }
        if listaban {
            for darab in sor.split('"').skip(1).step_by(2) {
                let darab = darab.trim();
                if darab.split("->").count() != 2 {
                    return Err(format!("hibás él a fuggesek.toml-ban: {darab}"));
                }
                let mut reszek = darab.split("->").map(str::trim);
                let (forras, cel) = (reszek.next().unwrap_or(""), reszek.next().unwrap_or(""));
                elek.insert(format!("{forras} -> {cel}"));
            }
        }
    }
    if talalt {
        Ok(elek)
    } else {
        Err("a fuggesek.toml-ból hiányzik az `engedett` lista".to_string())
    }
}

/// A `[package]` szakasz `name` értéke.
fn csomag_neve(manifest: &str) -> Option<String> {
    let mut csomagban = false;
    for sor in manifest.lines() {
        let sor = sor.trim();
        if sor.starts_with('[') {
            csomagban = sor == "[package]";
        } else if csomagban && sor.starts_with("name") {
            return sor.split('"').nth(1).map(str::to_string);
        }
    }
    None
}

fn toml_szoveg_mezo(ertek: &str, mezo: &str) -> Option<String> {
    ertek
        .trim()
        .trim_start_matches('{')
        .trim_end_matches('}')
        .split(',')
        .filter_map(|resz| resz.split_once('='))
        .find_map(|(kulcs, ertek)| {
            if kulcs.trim() != mezo {
                return None;
            }
            ertek
                .trim()
                .strip_prefix('"')
                .and_then(|ertek| ertek.strip_suffix('"'))
                .map(str::to_string)
        })
}

fn fuggoseg_szakasz(fejlec: &str) -> bool {
    fejlec
        .trim_matches(|c| c == '[' || c == ']')
        .contains("dependencies")
}

/// A gyökér `[workspace.dependencies]` szakaszának belső álnevei és csomagnevei.
fn gyoker_belso_nevek(gyoker_manifest: &str) -> BTreeMap<String, String> {
    let mut nevek = BTreeMap::new();
    let mut szakaszban = false;
    for sor in gyoker_manifest.lines() {
        let sor = sor.split('#').next().unwrap_or("").trim();
        if sor.starts_with('[') {
            szakaszban = sor == "[workspace.dependencies]";
        } else if szakaszban {
            if let Some((nev, _)) = sor.split_once('=') {
                let nev = nev.trim();
                let ertek = sor.split_once('=').map(|(_, ertek)| ertek).unwrap_or("");
                if toml_szoveg_mezo(ertek, "path").is_some() {
                    nevek.insert(
                        nev.to_string(),
                        toml_szoveg_mezo(ertek, "package").unwrap_or_else(|| nev.to_string()),
                    );
                }
            }
        }
    }
    nevek
}

/// Egy csomag belső függései (a függő csomagok nevei).
fn belso_fuggesek(manifest: &str, gyoker_belso: &BTreeMap<String, String>) -> BTreeSet<String> {
    let mut cel = BTreeSet::new();
    let mut szakasz_neve: Option<String> = None;
    let mut szakasz_csomagneve: Option<String> = None;
    let mut szakasz_belso = false;
    let mut szakasz_workspace = false;
    let mut fuggosegi = false;
    for sor in manifest.lines() {
        let sor = sor.split('#').next().unwrap_or("").trim();
        if sor.starts_with('[') {
            if let Some(nev) = szakasz_neve.take() {
                if szakasz_belso {
                    cel.insert(szakasz_csomagneve.unwrap_or(nev));
                }
            }
            fuggosegi = fuggoseg_szakasz(sor);
            // `[dependencies.pr-x]` alak: a név a fejlécben van.
            szakasz_neve = sor
                .trim_matches(|c| c == '[' || c == ']')
                .rsplit_once("dependencies.")
                .map(|(_, nev)| nev.trim().to_string());
            szakasz_csomagneve = None;
            szakasz_belso = false;
            szakasz_workspace = false;
            continue;
        }
        if !fuggosegi || sor.is_empty() {
            continue;
        }
        let Some((kulcs, ertek)) = sor.split_once('=') else {
            continue;
        };
        let kulcs = kulcs.trim();
        if let Some(nev) = &szakasz_neve {
            if kulcs == "package" {
                szakasz_csomagneve = toml_szoveg_mezo(sor, "package");
            }
            if kulcs == "path" {
                szakasz_belso = true;
            } else if kulcs == "workspace" && gyoker_belso.contains_key(nev) {
                szakasz_belso = true;
                szakasz_workspace = true;
            }
            continue;
        }
        let nev = kulcs.split('.').next().unwrap_or(kulcs).trim();
        let munkaterulet = ertek.contains("workspace") || kulcs.ends_with(".workspace");
        if ertek.contains("path") {
            cel.insert(toml_szoveg_mezo(ertek, "package").unwrap_or_else(|| nev.to_string()));
        } else if munkaterulet {
            if let Some(csomagnev) = toml_szoveg_mezo(ertek, "package") {
                if gyoker_belso.values().any(|belso| belso == &csomagnev) {
                    cel.insert(csomagnev);
                }
            } else if let Some(csomagnev) = gyoker_belso.get(nev) {
                cel.insert(csomagnev.clone());
            }
        }
    }
    if let Some(nev) = szakasz_neve {
        if szakasz_belso {
            if szakasz_workspace {
                if let Some(csomagnev) = szakasz_csomagneve {
                    if gyoker_belso.values().any(|belso| belso == &csomagnev) {
                        cel.insert(csomagnev);
                    }
                } else if let Some(csomagnev) = gyoker_belso.get(&nev) {
                    cel.insert(csomagnev.clone());
                }
            } else {
                cel.insert(szakasz_csomagneve.unwrap_or(nev));
            }
        }
    }
    cel
}

/// A listán kívüli élek.
fn tiltott_elek(
    csomagok: &[(String, BTreeSet<String>)],
    engedett: &BTreeSet<String>,
) -> Vec<String> {
    let mut tiltott = Vec::new();
    for (nev, celok) in csomagok {
        for cel in celok {
            let el = format!("{nev} -> {cel}");
            if !engedett.contains(&el) {
                tiltott.push(el);
            }
        }
    }
    tiltott
}

#[test]
fn a_munkaterulet_csomagjai_kozti_elek_a_fuggesek_toml_listajabol_valok() -> Result<(), String> {
    let gyoker = gyoker()?;
    let engedett = engedett_elek(&olvas(&gyoker.join("fuggesek.toml"))?)?;
    let belso = gyoker_belso_nevek(&olvas(&gyoker.join("Cargo.toml"))?);

    let mut csomagok = Vec::new();
    let mappa = fs::read_dir(gyoker.join("crates"))
        .map_err(|hiba| format!("a crates/ nem olvasható: {hiba}"))?;
    for bejegyzes in mappa {
        let ut = bejegyzes
            .map_err(|hiba| hiba.to_string())?
            .path()
            .join("Cargo.toml");
        if !ut.is_file() {
            continue;
        }
        let manifest = olvas(&ut)?;
        let nev = csomag_neve(&manifest)
            .ok_or_else(|| format!("{} nem nevez meg csomagot", ut.display()))?;
        csomagok.push((nev, belso_fuggesek(&manifest, &belso)));
    }

    let tiltott = tiltott_elek(&csomagok, &engedett);
    assert!(
        tiltott.is_empty(),
        "nem engedett csomagfüggések: {tiltott:?}"
    );
    Ok(())
}

#[test]
fn listan_kivuli_el_bukik() {
    let belso = BTreeMap::new();
    let manifest = "[package]\nname = \"pr-a\"\n[dependencies]\npr-b = { path = \"../pr-b\" }\n";
    let csomagok = vec![("pr-a".to_string(), belso_fuggesek(manifest, &belso))];
    assert_eq!(
        tiltott_elek(&csomagok, &BTreeSet::new()),
        vec!["pr-a -> pr-b".to_string()]
    );
}

#[test]
fn listan_levo_el_atmegy() -> Result<(), String> {
    let belso = BTreeMap::new();
    let manifest = "[dependencies]\npr-b = { path = \"../pr-b\" }\n";
    let csomagok = vec![("pr-a".to_string(), belso_fuggesek(manifest, &belso))];
    let engedett = engedett_elek("engedett = [\n  \"pr-a -> pr-b\", # indok\n]\n")?;
    assert!(tiltott_elek(&csomagok, &engedett).is_empty());
    Ok(())
}

#[test]
fn atnevezett_belso_fuggoseg_a_csomagnevet_hasznalja() {
    let belso = BTreeMap::new();
    let manifest =
        "[dependencies]\npr-alias = { package = \"pr-valodi\", path = \"../pr-valodi\" }\n";
    assert_eq!(
        belso_fuggesek(manifest, &belso),
        ["pr-valodi".to_string()].into()
    );

    let munkaterulet = gyoker_belso_nevek(
        "[workspace.dependencies]\npr-alias = { package = \"pr-valodi\", path = \"crates/pr-valodi\" }\n",
    );
    assert_eq!(
        belso_fuggesek("[dependencies]\npr-alias.workspace = true\n", &munkaterulet),
        ["pr-valodi".to_string()].into()
    );
}

#[test]
fn munkateruleti_fuggoseg_sorvegi_megjegyzessel_is_feloldodik() {
    let munkaterulet = gyoker_belso_nevek(
        "[workspace.dependencies]\npr-alias = { package = \"pr-valodi\", path = \"crates/pr-valodi\" } # belső függés\n",
    );
    assert_eq!(
        belso_fuggesek("[dependencies]\npr-alias.workspace = true\n", &munkaterulet),
        ["pr-valodi".to_string()].into()
    );
}

#[test]
fn tablaformaju_workspace_fuggoseg_a_csomagnevre_oldodik() {
    let munkaterulet = gyoker_belso_nevek(
        "[workspace.dependencies]\npr-alias = { package = \"pr-valodi\", path = \"crates/pr-valodi\" }\n",
    );
    let manifest = "[dependencies.pr-alias]\nworkspace = true\n";
    assert_eq!(
        belso_fuggesek(manifest, &munkaterulet),
        ["pr-valodi".to_string()].into()
    );
}

#[test]
fn kulso_package_nev_workspace_fuggosegnel_nem_belso_el() {
    let munkaterulet: BTreeMap<String, String> = [("pr-a".to_string(), "pr-a".to_string())].into();
    let manifest = "[dependencies]\nserde = { workspace = true, package = \"serde\" }\n";
    assert!(belso_fuggesek(manifest, &munkaterulet).is_empty());
}

#[test]
fn a_fejlesztoi_es_munkateruleti_el_is_el() {
    let belso: BTreeMap<String, String> = [("pr-c".to_string(), "pr-c".to_string())].into();
    let dev = "[dev-dependencies]\npr-b = { path = \"../pr-b\" }\n";
    assert_eq!(belso_fuggesek(dev, &belso), ["pr-b".to_string()].into());
    let szokoz = "[dependencies]\npr-c = { workspace = true }\nserde = { workspace = true }\n";
    assert_eq!(belso_fuggesek(szokoz, &belso), ["pr-c".to_string()].into());
    let pontos = "[dependencies]\npr-c.workspace = true\n";
    assert_eq!(belso_fuggesek(pontos, &belso), ["pr-c".to_string()].into());
    let fejlecben = "[dependencies.pr-d]\npath = \"../pr-d\"\n";
    assert_eq!(
        belso_fuggesek(fejlecben, &belso),
        ["pr-d".to_string()].into()
    );
}

#[test]
fn kulso_fuggoseg_nem_el() {
    let belso = BTreeMap::new();
    let manifest = "[dependencies]\nserde = \"=1.0.0\"\n";
    assert!(belso_fuggesek(manifest, &belso).is_empty());
}

#[test]
fn hibas_vagy_hianyzo_lista_hiba() {
    assert!(engedett_elek("nincs = 1\n").is_err());
    assert!(engedett_elek("engedett = [\n \"csak-egy\",\n]\n").is_err());
}
