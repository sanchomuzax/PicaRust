//! Mely képeken futott már le az arckeresés (hogy újraindítás után ne fusson újra).

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn kulcs(ut: &Path, meret: u64, mtime_ns: u128) -> String {
    format!("{}\t{meret}\t{mtime_ns}", ut.display())
}

/// A már feldolgozott képek kulcsai; hiányzó fájlnál üres.
pub fn beolvas(ut: &Path) -> HashSet<String> {
    std::fs::read_to_string(ut)
        .map(|s| {
            s.lines()
                .filter_map(|l| l.rsplit_once('\t').map(|(k, _)| k.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

pub fn hozzafuz(
    ut: &Path,
    kep_ut: &Path,
    meret: u64,
    mtime_ns: u128,
    arcok: usize,
) -> std::io::Result<()> {
    if let Some(mappa) = ut.parent() {
        std::fs::create_dir_all(mappa)?;
    }
    let mut fajl = OpenOptions::new().create(true).append(true).open(ut)?;
    writeln!(fajl, "{}\t{arcok}", kulcs(kep_ut, meret, mtime_ns))
}

#[cfg(test)]
mod tesztek {
    use super::*;

    #[test]
    fn a_hozzafuzott_kep_visszaolvashato() -> Result<(), String> {
        let munkafa = Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| "nem talalhato a munkafa gyokere".to_string())?;
        let ut = munkafa
            .join(".tmp")
            .join(format!("picarust-arcok-{}.tsv", std::process::id()));
        let _ = std::fs::remove_file(&ut);
        assert!(beolvas(&ut).is_empty());
        let kep_ut = Path::new("a b.jpg");
        hozzafuz(&ut, kep_ut, 7, 9, 3).map_err(|e| e.to_string())?;
        assert!(beolvas(&ut).contains(&kulcs(kep_ut, 7, 9)));
        let _ = std::fs::remove_file(&ut);
        Ok(())
    }
}
