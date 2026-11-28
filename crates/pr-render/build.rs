use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(hiba) = futtat() {
        eprintln!("Az effektjegyzék építése sikertelen: {hiba}");
        std::process::exit(1);
    }
}

fn futtat() -> Result<(), Box<dyn Error>> {
    let manifestum = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let forras = manifestum.join("src/effektek");
    println!("cargo:rerun-if-changed={}", forras.display());

    let mut fajlok = Vec::new();
    match fs::read_dir(&forras) {
        Ok(bejegyzesek) => {
            for bejegyzes in bejegyzesek {
                let ut = bejegyzes?.path();
                if ut
                    .extension()
                    .is_some_and(|kiterjesztes| kiterjesztes == "rs")
                {
                    println!("cargo:rerun-if-changed={}", ut.display());
                    fajlok.push(ut);
                }
            }
        }
        Err(hiba) if hiba.kind() == std::io::ErrorKind::NotFound => {}
        Err(hiba) => return Err(Box::new(hiba)),
    }
    fajlok.sort();
    let modul_nevek: Vec<String> = fajlok
        .iter()
        .map(|fajl| modul_nev(fajl))
        .collect::<Result<_, _>>()?;

    let mut kimenet = String::from("pub(crate) mod effektek {\n");
    for (fajl, modul) in fajlok.iter().zip(&modul_nevek) {
        let include_ut = rust_ut_literal(fajl);
        kimenet.push_str(&format!("    mod {modul} {{ include!({include_ut}); }}\n"));
    }
    kimenet.push_str(
        "    pub(crate) static JEGYZEK: &[(&str, &str, usize, crate::EffektAlkalmazo)] = &[\n",
    );
    for modul in &modul_nevek {
        kimenet.push_str(&format!(
            "        ({modul}::NEV, {modul}::LANC_NEV, {modul}::PARAMETER_COUNT, {modul}::alkalmaz),\n"
        ));
    }
    kimenet.push_str("    ];\n}\n");

    let cel = PathBuf::from(std::env::var("OUT_DIR")?).join("effektek.rs");
    fs::write(cel, kimenet)?;
    Ok(())
}

fn rust_ut_literal(ut: &Path) -> String {
    format!("{:?}", ut.to_string_lossy())
}

fn modul_nev(ut: &Path) -> Result<String, std::io::Error> {
    let stem = ut
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| {
            std::io::Error::other(format!("Nem használható effektfájlnév: {}", ut.display()))
        })?;
    if !stem
        .chars()
        .all(|karakter| karakter.is_ascii_alphanumeric() || karakter == '_')
    {
        return Err(std::io::Error::other(format!(
            "Az effektfájlnév nem alakítható Rust-modullá: {}",
            ut.display()
        )));
    }
    Ok(format!("_effekt_{stem}"))
}
