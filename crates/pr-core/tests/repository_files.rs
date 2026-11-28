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
            ci.contains(nev),
            "hiányzó ellenőrzésnév a ci.yml-ben: {nev}"
        );
    }
    let kapu = fs::read_to_string(gyoker.join(".github/workflows/szerzo-kapu.yml"))
        .map_err(|hiba| hiba.to_string())?;
    assert!(
        kapu.contains("name: szerzo"),
        "a kapu ellenőrzés-neve `szerzo`"
    );
    Ok(())
}
