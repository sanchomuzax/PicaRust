//! A repó gyökerének kötelező fájljai megvannak, és a `.gitignore` tartalmazza
//! a titokmintákat.

use std::fs;
use std::path::{Path, PathBuf};

const KOTELEZO_FAJLOK: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "deny.toml",
    "fuggesek.toml",
    ".gitignore",
    "README.md",
    "CONTRIBUTING.md",
    "CLAUDE.md",
    "AGENTS.md",
    "CHANGELOG.md",
    "changelog.d/README.md",
    ".github/ISSUE_TEMPLATE/hiba.md",
    ".github/workflows/ci.yml",
    ".github/workflows/szerzo-kapu.yml",
    ".github/workflows/kiadasi-or.yml",
    ".claude/settings.json",
    "crates/pr-core/Cargo.toml",
    "crates/pr-core/src/lib.rs",
];

/// A `.gitignore` sorai, amelyek nélkül titok kerülhet a repóba.
const TITOKMINTAK: &[&str] = &[
    ".env*",
    "*.pem",
    "*.key",
    "id_rsa*",
    "*.p12",
    "credentials*.json",
    "token*.json",
];

/// A `.gitignore` sorai, amelyek az eszközök és a build állapotát tartják ki.
const ALLAPOTMINTAK: &[&str] = &[
    "target/",
    ".ell/",
    ".bt/",
    ".tmp/",
    ".codex-*",
    ".gemini-*",
    ".ellenor-*",
];

fn gyoker() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| "a pr-core nem a munkaterület alatt van".to_string())
}

fn gitignore_sorai() -> Result<Vec<String>, String> {
    let szoveg = fs::read_to_string(gyoker()?.join(".gitignore"))
        .map_err(|hiba| format!(".gitignore nem olvasható: {hiba}"))?;
    Ok(szoveg.lines().map(|sor| sor.trim().to_string()).collect())
}

fn ci_munkanevek(szoveg: &str) -> Vec<String> {
    let mut munkakban = false;
    let mut munkaban = false;
    let mut nevek = Vec::new();
    for sor in szoveg.lines() {
        let tartalom = sor.split('#').next().unwrap_or("").trim_end();
        let behuzas = tartalom.len() - tartalom.trim_start().len();
        let tartalom = tartalom.trim();
        if tartalom.is_empty() {
            continue;
        }
        if behuzas == 0 {
            munkakban = tartalom == "jobs:";
            munkaban = false;
            continue;
        }
        if !munkakban {
            continue;
        }
        if behuzas == 2 && tartalom.ends_with(':') {
            munkaban = true;
            continue;
        }
        if behuzas == 2 {
            munkaban = false;
        }
        if behuzas == 4 && munkaban {
            if let Some((kulcs, ertek)) = tartalom.split_once(':') {
                if kulcs.trim() == "name" {
                    nevek.push(
                        ertek
                            .trim()
                            .trim_matches(|karakter| karakter == '"' || karakter == '\'')
                            .to_string(),
                    );
                }
            }
        }
    }
    nevek
}

#[test]
fn a_gyoker_kotelezo_fajljai_megvannak() -> Result<(), String> {
    let gyoker = gyoker()?;
    let hianyzo: Vec<&str> = KOTELEZO_FAJLOK
        .iter()
        .copied()
        .filter(|ut| !gyoker.join(ut).is_file())
        .collect();
    assert!(hianyzo.is_empty(), "hiányzó fájlok: {hianyzo:?}");
    Ok(())
}

#[test]
fn a_gitignore_tartalmazza_a_titokmintakat() -> Result<(), String> {
    let sorok = gitignore_sorai()?;
    let hianyzo: Vec<&str> = TITOKMINTAK
        .iter()
        .copied()
        .filter(|minta| !sorok.iter().any(|sor| sor == minta))
        .collect();
    assert!(hianyzo.is_empty(), "hiányzó titokminták: {hianyzo:?}");
    Ok(())
}

#[test]
fn a_gitignore_kizarja_az_eszkoz_es_build_allapotot() -> Result<(), String> {
    let sorok = gitignore_sorai()?;
    let hianyzo: Vec<&str> = ALLAPOTMINTAK
        .iter()
        .copied()
        .filter(|minta| !sorok.iter().any(|sor| sor == minta))
        .collect();
    assert!(hianyzo.is_empty(), "hiányzó állapotminták: {hianyzo:?}");
    Ok(())
}

#[test]
fn a_ci_ellenorzesek_neve_rogzitett() -> Result<(), String> {
    let gyoker = gyoker()?;
    let ci = fs::read_to_string(gyoker.join(".github/workflows/ci.yml"))
        .map_err(|hiba| hiba.to_string())?;
    for nev in ["teszt (ubuntu-latest)", "csomag (windows-latest)"] {
        assert!(
            ci_munkanevek(&ci).iter().any(|munkanev| munkanev == nev),
            "hiányzó ellenőrzésnév a ci.yml-ben: {nev}"
        );
    }
    let kapu = fs::read_to_string(gyoker.join(".github/workflows/szerzo-kapu.yml"))
        .map_err(|hiba| hiba.to_string())?;
    assert!(
        ci_munkanevek(&kapu)
            .iter()
            .any(|munkanev| munkanev == "szerzo"),
        "a kapu ellenőrzés-neve `szerzo`"
    );
    Ok(())
}

#[test]
fn a_ci_megjegyzesben_levo_nev_nem_munkanev() {
    let yaml = "# jobs:\n#   teszt:\n#     name: teszt (ubuntu-latest)\njobs:\n  ellenorzes:\n    runs-on: ubuntu-latest\n";
    assert!(ci_munkanevek(yaml).is_empty());
}

#[test]
fn a_ci_munkaszintu_idezojeles_nevet_megtalalja() {
    let yaml =
        "jobs:\n  ellenorzes:\n    name: \"teszt (ubuntu-latest)\"\n    runs-on: ubuntu-latest\n";
    assert_eq!(ci_munkanevek(yaml), ["teszt (ubuntu-latest)".to_string()]);
}

#[test]
fn a_ci_lepesszintu_nev_nem_munkanev() {
    let yaml = "jobs:\n  ellenorzes:\n    runs-on: ubuntu-latest\n    steps:\n      - name: teszt (ubuntu-latest)\n        run: cargo test\n";
    assert!(ci_munkanevek(yaml).is_empty());
}
