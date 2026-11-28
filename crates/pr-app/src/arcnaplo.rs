//! Mely képeken futott már le az arckeresés (hogy újraindítás után ne fusson újra).

use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::konyvtar::Foto;

pub fn kulcs(foto: &Foto) -> String {
    format!("{}\t{}\t{}", foto.ut.display(), foto.meret, foto.mtime_ns)
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

pub fn hozzafuz(ut: &Path, foto: &Foto, arcok: usize) -> std::io::Result<()> {
    if let Some(mappa) = ut.parent() {
        std::fs::create_dir_all(mappa)?;
    }
    let mut fajl = OpenOptions::new().create(true).append(true).open(ut)?;
    writeln!(fajl, "{}\t{arcok}", kulcs(foto))
}

#[cfg(test)]
mod tesztek {
    use super::*;

    #[test]
    fn a_hozzafuzott_kep_visszaolvashato() -> Result<(), String> {
        let ut = std::env::temp_dir().join(format!("picarust-arcok-{}.tsv", std::process::id()));
        let _ = std::fs::remove_file(&ut);
        assert!(beolvas(&ut).is_empty());
        let foto = Foto {
            ut: "a b.jpg".into(),
            meret: 7,
            mtime_ns: 9,
        };
        hozzafuz(&ut, &foto, 3).map_err(|e| e.to_string())?;
        assert!(beolvas(&ut).contains(&kulcs(&foto)));
        let _ = std::fs::remove_file(&ut);
        Ok(())
    }
}
