//! A PicaRust saját SQLite indexe.
//!
//! Az SQLite sémát kizárólag ez a csomag hozza létre és módosítja; a többi csomag csak a nyilvános
//! lekérdezéseit hívja. Az index bármikor újraépíthető a képtárból, a képtárba nem ír.
//! Az útvonalak tárolása és összehasonlítása bájtra pontos; a hívónak ugyanazt az alakot kell
//! átadnia, amelyet a bejárás adott.
#![forbid(unsafe_code)]

use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use pr_scan::Mappa;
use rusqlite::{params, Connection, Transaction};

const SHEMA_VERZIO: i64 = 1;

/// Egy indexelt mappa összefoglaló sora.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MappaSor {
    pub ut: PathBuf,
    pub nev: String,
    pub kep_db: u64,
}

/// Egy indexelt kép adatai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KepSor {
    pub ut: PathBuf,
    pub mappa: PathBuf,
    pub meret: u64,
    pub mtime_ns: u128,
}

/// A legutóbbi teljes frissítés eredménye.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Osszesites {
    pub mappak: u64,
    pub kepek: u64,
    pub uj: u64,
    pub modosult: u64,
    pub torolt: u64,
}

/// Az index megnyitásának és lekérdezésének hibái.
#[derive(Debug)]
pub enum Hiba {
    Megnyitas(String),
    Schema(String),
    Lekerdezes(String),
}

impl fmt::Display for Hiba {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Megnyitas(reszlet) => write!(formatter, "Az index nem nyitható meg: {reszlet}"),
            Self::Schema(reszlet) => write!(formatter, "Az index sémája hibás: {reszlet}"),
            Self::Lekerdezes(reszlet) => {
                write!(formatter, "Az index lekérdezése sikertelen: {reszlet}")
            }
        }
    }
}

impl Error for Hiba {}

/// Az index adatbáziskapcsolatát birtokló objektum.
pub struct Index {
    kapcsolat: Connection,
}

#[derive(Clone)]
struct KepAdat {
    mappa: PathBuf,
    nev_keres: String,
    meret: u64,
    mtime_ns: u128,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SchemaAllapot {
    Ures,
    Ervenyes,
    Ismeretlen,
    AdatbazisSerult,
}

impl Index {
    /// Megnyitja vagy létrehozza az adatbázist. Sérült vagy ismeretlen sémájú fájlt félretesz.
    pub fn megnyit(fajl: &Path) -> Result<Self, Hiba> {
        if let Some(szulo) = fajl.parent().filter(|ut| !ut.as_os_str().is_empty()) {
            std::fs::create_dir_all(szulo).map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
        }

        let mut kapcsolat = match Connection::open(fajl) {
            Ok(kapcsolat) => kapcsolat,
            Err(hiba) if sqlite_szerint_serult(&hiba) => {
                regi_felretesz(fajl)?;
                Connection::open(fajl).map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?
            }
            Err(hiba) => return Err(Hiba::Megnyitas(hiba.to_string())),
        };
        kapcsolat
            .busy_timeout(Duration::from_secs(3))
            .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
        let allapot = match schema_allapot(&kapcsolat) {
            Ok(allapot) => allapot,
            Err(hiba) if sqlite_szerint_serult(&hiba) => SchemaAllapot::AdatbazisSerult,
            Err(hiba) => return Err(Hiba::Megnyitas(hiba.to_string())),
        };
        let kapcsolat = match allapot {
            SchemaAllapot::Ervenyes => kapcsolat,
            SchemaAllapot::Ures => {
                schema_letrehoz(&mut kapcsolat)?;
                kapcsolat
            }
            SchemaAllapot::Ismeretlen | SchemaAllapot::AdatbazisSerult => {
                drop(kapcsolat);
                regi_felretesz(fajl)?;
                let mut kapcsolat =
                    Connection::open(fajl).map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
                kapcsolat
                    .busy_timeout(Duration::from_secs(3))
                    .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
                schema_letrehoz(&mut kapcsolat)?;
                kapcsolat
            }
        };
        kapcsolat
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
        Ok(Self { kapcsolat })
    }

    /// A kapott bejárást egy tranzakcióban az index teljes tartalmává teszi.
    pub fn frissit(&mut self, mappak: &[Mappa]) -> Result<Osszesites, Hiba> {
        let mut cel_mappak = BTreeMap::<Vec<u8>, String>::new();
        let mut cel_kepek = BTreeMap::<Vec<u8>, KepAdat>::new();
        for mappa in mappak {
            cel_mappak.insert(ut_bajtok(&mappa.ut), mappa.nev.clone());
            for foto in &mappa.fotok {
                cel_kepek.insert(
                    ut_bajtok(&foto.ut),
                    KepAdat {
                        mappa: mappa.ut.clone(),
                        nev_keres: ekezet_nelkul(
                            foto.ut
                                .file_name()
                                .unwrap_or(foto.ut.as_os_str())
                                .to_string_lossy()
                                .as_ref(),
                        ),
                        meret: foto.meret,
                        mtime_ns: foto.mtime_ns,
                    },
                );
            }
        }

        let tx = self
            .kapcsolat
            .transaction()
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        let regi_kepek = regi_kepek(&tx)?;
        let mut uj = 0_u64;
        let mut modosult = 0_u64;
        for (kulcs, adat) in &cel_kepek {
            match regi_kepek.get(kulcs) {
                None => uj += 1,
                Some(regi)
                    if regi.mappa != ut_bajtok(&adat.mappa)
                        || regi.meret != adat.meret.to_string()
                        || regi.mtime_ns != adat.mtime_ns.to_string() =>
                {
                    modosult += 1;
                }
                Some(_) => {}
            }
        }
        let torolt = regi_kepek
            .keys()
            .filter(|kulcs| !cel_kepek.contains_key(*kulcs))
            .count() as u64;

        tx.execute("DELETE FROM mappak", [])
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        mappak_beszur(&tx, &cel_mappak)?;
        kepek_beszur(&tx, &cel_kepek)?;
        tx.commit()
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;

        Ok(Osszesites {
            mappak: cel_mappak.len() as u64,
            kepek: cel_kepek.len() as u64,
            uj,
            modosult,
            torolt,
        })
    }

    /// Az indexelt mappákat útvonal szerint rendezve adja vissza.
    pub fn mappak(&self) -> Result<Vec<MappaSor>, Hiba> {
        let mut statement = self
            .kapcsolat
            .prepare(
                "SELECT m.ut, m.nev, COUNT(k.ut) \
                 FROM mappak AS m LEFT JOIN kepek AS k ON k.mappa = m.ut \
                 GROUP BY m.ut, m.nev",
            )
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        let sorok = statement
            .query_map([], |sor| {
                Ok((
                    sor.get::<_, Vec<u8>>(0)?,
                    sor.get::<_, String>(1)?,
                    sor.get::<_, i64>(2)?,
                ))
            })
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        let mut mappak = Vec::new();
        for sor in sorok {
            let (ut, nev, kep_db) = sor.map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
            let kep_db = u64::try_from(kep_db).map_err(|hiba| Hiba::Schema(hiba.to_string()))?;
            mappak.push(MappaSor {
                ut: ut_betoltes(&ut)?,
                nev,
                kep_db,
            });
        }
        mappak.sort_by(|bal, jobb| bal.ut.cmp(&jobb.ut));
        Ok(mappak)
    }

    /// Egy mappa képeit útvonal szerint rendezve adja vissza.
    pub fn kepek(&self, mappa: &Path) -> Result<Vec<KepSor>, Hiba> {
        let bajtok = ut_bajtok(mappa);
        let mut kepek = kep_lista(
            &self.kapcsolat,
            "SELECT ut, mappa, meret, mtime_ns FROM kepek WHERE mappa = ?1",
            params![bajtok],
        )?;
        kepek.sort_by(|bal, jobb| bal.ut.cmp(&jobb.ut));
        Ok(kepek)
    }

    /// Fájlnévben keres, kis- és nagybetűre, valamint ékezetre érzéketlenül.
    pub fn keres(&self, szo: &str) -> Result<Vec<KepSor>, Hiba> {
        let szo = szo.trim();
        if szo.is_empty() {
            return Ok(Vec::new());
        }
        kep_lista(
            &self.kapcsolat,
            "SELECT ut, mappa, meret, mtime_ns FROM kepek \
             WHERE instr(nev_keres, ?1) > 0",
            params![ekezet_nelkul(szo)],
        )
    }
}

fn schema_allapot(kapcsolat: &Connection) -> Result<SchemaAllapot, rusqlite::Error> {
    let verzio = kapcsolat.query_row("PRAGMA user_version", [], |sor| sor.get::<_, i64>(0))?;
    if verzio == 0 {
        let tablakszam = kapcsolat.query_row(
            "SELECT COUNT(*) FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            [],
            |sor| sor.get::<_, i64>(0),
        )?;
        return Ok(if tablakszam == 0 {
            SchemaAllapot::Ures
        } else {
            SchemaAllapot::Ismeretlen
        });
    }
    if verzio != SHEMA_VERZIO {
        return Ok(SchemaAllapot::Ismeretlen);
    }
    kapcsolat.prepare("SELECT ut, nev FROM mappak LIMIT 0")?;
    kapcsolat.prepare("SELECT ut, mappa, nev_keres, meret, mtime_ns FROM kepek LIMIT 0")?;
    Ok(SchemaAllapot::Ervenyes)
}

fn sqlite_szerint_serult(hiba: &rusqlite::Error) -> bool {
    matches!(
        hiba,
        rusqlite::Error::SqliteFailure(sqlite, _)
            if matches!(
                sqlite.code,
                rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
            )
    )
}

fn schema_letrehoz(kapcsolat: &mut Connection) -> Result<(), Hiba> {
    let tx = kapcsolat
        .transaction()
        .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
    tx.execute_batch(
        "CREATE TABLE mappak (
            ut BLOB PRIMARY KEY NOT NULL,
            nev TEXT NOT NULL
         );
         CREATE TABLE kepek (
            ut BLOB PRIMARY KEY NOT NULL,
            mappa BLOB NOT NULL REFERENCES mappak(ut) ON DELETE CASCADE,
            nev_keres TEXT NOT NULL,
            meret TEXT NOT NULL,
            mtime_ns TEXT NOT NULL
         );
         CREATE INDEX kepek_mappa ON kepek(mappa);
         PRAGMA user_version = 1;",
    )
    .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))?;
    tx.commit()
        .map_err(|hiba| Hiba::Megnyitas(hiba.to_string()))
}

fn regi_felretesz(fajl: &Path) -> Result<(), Hiba> {
    let mut alap = fajl.as_os_str().to_os_string();
    alap.push(".regi");
    let mut cel = PathBuf::from(&alap);
    let mut sorszam = 1_u64;
    while csalad_letezik(&cel) {
        let mut nev = alap.clone();
        nev.push(format!(".{sorszam}"));
        cel = PathBuf::from(nev);
        sorszam = sorszam
            .checked_add(1)
            .ok_or_else(|| Hiba::Megnyitas("elfogytak a félretett fájlnevek".to_string()))?;
    }
    let mut parok = Vec::new();
    for kiegeszito in ["-journal", "-wal", "-shm"] {
        parok.push((ut_suffixel(fajl, kiegeszito), ut_suffixel(&cel, kiegeszito)));
    }
    parok.push((fajl.to_path_buf(), cel));
    let mut atmozgatott = Vec::new();
    for (eredeti, regi) in parok {
        if !eredeti.exists() {
            continue;
        }
        if let Err(hiba) = std::fs::rename(&eredeti, &regi) {
            for (forras, cel) in atmozgatott.iter().rev() {
                let _ = std::fs::rename(cel, forras);
            }
            return Err(Hiba::Megnyitas(hiba.to_string()));
        }
        atmozgatott.push((eredeti, regi));
    }
    Ok(())
}

fn csalad_letezik(fajl: &Path) -> bool {
    fajl.exists()
        || ["-journal", "-wal", "-shm"]
            .iter()
            .any(|kiegeszito| ut_suffixel(fajl, kiegeszito).exists())
}

fn ut_suffixel(ut: &Path, kiegeszito: &str) -> PathBuf {
    let mut nev = ut.as_os_str().to_os_string();
    nev.push(kiegeszito);
    PathBuf::from(nev)
}

fn regi_kepek(tx: &Transaction<'_>) -> Result<HashMap<Vec<u8>, RegiKep>, Hiba> {
    let mut statement = tx
        .prepare("SELECT ut, mappa, meret, mtime_ns FROM kepek")
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    let sorok = statement
        .query_map([], |sor| {
            Ok((
                sor.get::<_, Vec<u8>>(0)?,
                sor.get::<_, Vec<u8>>(1)?,
                sor.get::<_, String>(2)?,
                sor.get::<_, String>(3)?,
            ))
        })
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    let mut kepek = HashMap::new();
    for sor in sorok {
        let (ut, mappa, meret, mtime_ns) =
            sor.map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        kepek.insert(
            ut,
            RegiKep {
                mappa,
                meret,
                mtime_ns,
            },
        );
    }
    Ok(kepek)
}

struct RegiKep {
    mappa: Vec<u8>,
    meret: String,
    mtime_ns: String,
}

fn mappak_beszur(tx: &Transaction<'_>, mappak: &BTreeMap<Vec<u8>, String>) -> Result<(), Hiba> {
    let mut statement = tx
        .prepare("INSERT INTO mappak (ut, nev) VALUES (?1, ?2)")
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    for (ut, nev) in mappak {
        statement
            .execute(params![ut, nev])
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    }
    Ok(())
}

fn kepek_beszur(tx: &Transaction<'_>, kepek: &BTreeMap<Vec<u8>, KepAdat>) -> Result<(), Hiba> {
    let mut statement = tx
        .prepare(
            "INSERT INTO kepek (ut, mappa, nev_keres, meret, mtime_ns) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    for (ut, adat) in kepek {
        statement
            .execute(params![
                ut,
                ut_bajtok(&adat.mappa),
                adat.nev_keres,
                adat.meret.to_string(),
                adat.mtime_ns.to_string()
            ])
            .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    }
    Ok(())
}

fn kep_lista<P: rusqlite::Params>(
    kapcsolat: &Connection,
    lekerdezes: &str,
    parameterek: P,
) -> Result<Vec<KepSor>, Hiba> {
    let mut statement = kapcsolat
        .prepare(lekerdezes)
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    let sorok = statement
        .query_map(parameterek, |sor| {
            Ok((
                sor.get::<_, Vec<u8>>(0)?,
                sor.get::<_, Vec<u8>>(1)?,
                sor.get::<_, String>(2)?,
                sor.get::<_, String>(3)?,
            ))
        })
        .map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
    let mut kepek = Vec::new();
    for sor in sorok {
        let (ut, mappa, meret, mtime_ns) =
            sor.map_err(|hiba| Hiba::Lekerdezes(hiba.to_string()))?;
        let meret = meret
            .parse::<u64>()
            .map_err(|hiba| Hiba::Schema(hiba.to_string()))?;
        let mtime_ns = mtime_ns
            .parse::<u128>()
            .map_err(|hiba| Hiba::Schema(hiba.to_string()))?;
        kepek.push(KepSor {
            ut: ut_betoltes(&ut)?,
            mappa: ut_betoltes(&mappa)?,
            meret,
            mtime_ns,
        });
    }
    kepek.sort_by(|bal, jobb| bal.ut.cmp(&jobb.ut));
    Ok(kepek)
}

fn ut_bajtok(ut: &Path) -> Vec<u8> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        ut.as_os_str().as_bytes().to_vec()
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        ut.as_os_str()
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect()
    }
    #[cfg(not(any(unix, windows)))]
    {
        ut.to_string_lossy().as_bytes().to_vec()
    }
}

fn ut_betoltes(bajtok: &[u8]) -> Result<PathBuf, Hiba> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        Ok(PathBuf::from(std::ffi::OsString::from_vec(bajtok.to_vec())))
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let szavak = bajtok
            .chunks_exact(2)
            .map(|par| u16::from_le_bytes([par[0], par[1]]))
            .collect::<Vec<_>>();
        if bajtok.len() % 2 != 0 {
            return Err(Hiba::Schema("hibás útvonaladat".to_string()));
        }
        Ok(PathBuf::from(std::ffi::OsString::from_wide(&szavak)))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let szoveg =
            String::from_utf8(bajtok.to_vec()).map_err(|hiba| Hiba::Schema(hiba.to_string()))?;
        Ok(PathBuf::from(szoveg))
    }
}

fn ekezet_nelkul(szoveg: &str) -> String {
    let mut eredmeny = String::new();
    for karakter in szoveg.chars().flat_map(char::to_lowercase) {
        let alap = match karakter {
            'á' => "a",
            'é' => "e",
            'í' => "i",
            'ó' | 'ö' | 'ő' => "o",
            'ú' | 'ü' | 'ű' => "u",
            _ => {
                eredmeny.push(karakter);
                continue;
            }
        };
        eredmeny.push_str(alap);
    }
    eredmeny
}
