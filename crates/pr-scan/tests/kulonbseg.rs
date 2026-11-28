use std::path::PathBuf;

use pr_scan::{kulonbseg, Foto, Mappa};

fn foto(ut: &str, meret: u64, mtime_ns: u128) -> Foto {
    Foto {
        ut: PathBuf::from(ut),
        meret,
        mtime_ns,
    }
}

fn mappa(ut: &str, fotok: Vec<Foto>) -> Mappa {
    let ut_buf = PathBuf::from(ut);
    let nev = match ut_buf.file_name() {
        Some(n) => n.to_string_lossy().into_owned(),
        None => ut.to_string(),
    };
    Mappa {
        ut: ut_buf,
        nev,
        fotok,
    }
}

#[test]
fn uj_mappa() {
    let regi = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];
    let uj = vec![
        mappa(
            "keptar/2026-01",
            vec![foto("keptar/2026-01/a.jpg", 100, 10)],
        ),
        mappa(
            "keptar/2026-02",
            vec![foto("keptar/2026-02/b.jpg", 200, 20)],
        ),
    ];

    let valtozasok = kulonbseg(&regi, &uj);

    assert_eq!(valtozasok.uj_mappak, vec![PathBuf::from("keptar/2026-02")]);
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert_eq!(
        valtozasok.uj_kepek,
        vec![PathBuf::from("keptar/2026-02/b.jpg")]
    );
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert!(valtozasok.modosult_kepek.is_empty());
    assert!(!valtozasok.ures());
}

#[test]
fn megszunt_mappa() {
    let regi = vec![
        mappa(
            "keptar/2026-01",
            vec![foto("keptar/2026-01/a.jpg", 100, 10)],
        ),
        mappa(
            "keptar/2026-02",
            vec![foto("keptar/2026-02/b.jpg", 200, 20)],
        ),
    ];
    let uj = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.uj_mappak.is_empty());
    assert_eq!(
        valtozasok.megszunt_mappak,
        vec![PathBuf::from("keptar/2026-02")]
    );
    assert!(valtozasok.uj_kepek.is_empty());
    assert_eq!(
        valtozasok.megszunt_kepek,
        vec![PathBuf::from("keptar/2026-02/b.jpg")]
    );
    assert!(valtozasok.modosult_kepek.is_empty());
    assert!(!valtozasok.ures());
}

#[test]
fn uj_kep_meglevo_mappaban() {
    let regi = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];
    let uj = vec![mappa(
        "keptar/2026-01",
        vec![
            foto("keptar/2026-01/a.jpg", 100, 10),
            foto("keptar/2026-01/b.jpg", 150, 12),
        ],
    )];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert_eq!(
        valtozasok.uj_kepek,
        vec![PathBuf::from("keptar/2026-01/b.jpg")]
    );
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert!(valtozasok.modosult_kepek.is_empty());
    assert!(!valtozasok.ures());
}

#[test]
fn megszunt_kep() {
    let regi = vec![mappa(
        "keptar/2026-01",
        vec![
            foto("keptar/2026-01/a.jpg", 100, 10),
            foto("keptar/2026-01/b.jpg", 150, 12),
        ],
    )];
    let uj = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert!(valtozasok.uj_kepek.is_empty());
    assert_eq!(
        valtozasok.megszunt_kepek,
        vec![PathBuf::from("keptar/2026-01/b.jpg")]
    );
    assert!(valtozasok.modosult_kepek.is_empty());
    assert!(!valtozasok.ures());
}

#[test]
fn modosult_meret() {
    let regi = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];
    let uj = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 200, 10)],
    )];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert!(valtozasok.uj_kepek.is_empty());
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert_eq!(
        valtozasok.modosult_kepek,
        vec![PathBuf::from("keptar/2026-01/a.jpg")]
    );
    assert!(!valtozasok.ures());
}

#[test]
fn modosult_ido() {
    let regi = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 10)],
    )];
    let uj = vec![mappa(
        "keptar/2026-01",
        vec![foto("keptar/2026-01/a.jpg", 100, 20)],
    )];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert!(valtozasok.uj_kepek.is_empty());
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert_eq!(
        valtozasok.modosult_kepek,
        vec![PathBuf::from("keptar/2026-01/a.jpg")]
    );
    assert!(!valtozasok.ures());
}

#[test]
fn azonos_bemenet_ures_eredmenyt_ad() {
    let regi = vec![
        mappa(
            "keptar/2026-01",
            vec![foto("keptar/2026-01/a.jpg", 100, 10)],
        ),
        mappa(
            "keptar/2026-02",
            vec![foto("keptar/2026-02/b.jpg", 200, 20)],
        ),
    ];
    let uj = vec![
        mappa(
            "keptar/2026-01",
            vec![foto("keptar/2026-01/a.jpg", 100, 10)],
        ),
        mappa(
            "keptar/2026-02",
            vec![foto("keptar/2026-02/b.jpg", 200, 20)],
        ),
    ];

    let valtozasok = kulonbseg(&regi, &uj);

    assert!(valtozasok.ures());
    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert!(valtozasok.uj_kepek.is_empty());
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert!(valtozasok.modosult_kepek.is_empty());
}

#[test]
fn a_bemenet_megkevert_sorrendje_ugyanazt_az_eredmenyt_adja() {
    let regi_rendezett = vec![
        mappa(
            "keptar/a",
            vec![
                foto("keptar/a/1.jpg", 100, 10),
                foto("keptar/a/2.jpg", 200, 20),
            ],
        ),
        mappa("keptar/b", vec![foto("keptar/b/3.jpg", 300, 30)]),
        mappa("keptar/c", vec![foto("keptar/c/4.jpg", 400, 40)]),
    ];
    let uj_rendezett = vec![
        mappa(
            "keptar/a",
            vec![
                foto("keptar/a/1.jpg", 100, 10),
                foto("keptar/a/2.jpg", 999, 20), // modosult meret
                foto("keptar/a/5.jpg", 500, 50), // uj kep
            ],
        ),
        mappa(
            "keptar/c",
            vec![foto("keptar/c/4.jpg", 400, 99)], // modosult ido
        ),
        mappa(
            "keptar/d",
            vec![foto("keptar/d/6.jpg", 600, 60)], // uj mappa es uj kep
        ),
        // keptar/b megszunt (es keptar/b/3.jpg megszunt)
    ];

    let regi_kevert = vec![
        mappa("keptar/c", vec![foto("keptar/c/4.jpg", 400, 40)]),
        mappa(
            "keptar/a",
            vec![
                foto("keptar/a/2.jpg", 200, 20),
                foto("keptar/a/1.jpg", 100, 10),
            ],
        ),
        mappa("keptar/b", vec![foto("keptar/b/3.jpg", 300, 30)]),
    ];
    let uj_kevert = vec![
        mappa("keptar/d", vec![foto("keptar/d/6.jpg", 600, 60)]),
        mappa("keptar/c", vec![foto("keptar/c/4.jpg", 400, 99)]),
        mappa(
            "keptar/a",
            vec![
                foto("keptar/a/5.jpg", 500, 50),
                foto("keptar/a/1.jpg", 100, 10),
                foto("keptar/a/2.jpg", 999, 20),
            ],
        ),
    ];

    let v1 = kulonbseg(&regi_rendezett, &uj_rendezett);
    let v2 = kulonbseg(&regi_kevert, &uj_kevert);

    assert_eq!(v1, v2);

    // Ellenorizzuk a rendezettseget is:
    assert_eq!(v1.uj_mappak, vec![PathBuf::from("keptar/d")]);
    assert_eq!(v1.megszunt_mappak, vec![PathBuf::from("keptar/b")]);
    assert_eq!(
        v1.uj_kepek,
        vec![
            PathBuf::from("keptar/a/5.jpg"),
            PathBuf::from("keptar/d/6.jpg"),
        ]
    );
    assert_eq!(v1.megszunt_kepek, vec![PathBuf::from("keptar/b/3.jpg")]);
    assert_eq!(
        v1.modosult_kepek,
        vec![
            PathBuf::from("keptar/a/2.jpg"),
            PathBuf::from("keptar/c/4.jpg"),
        ]
    );
    assert!(!v1.ures());
}

#[test]
fn ures_bemenetek() {
    let valtozasok = kulonbseg(&[], &[]);

    assert!(valtozasok.ures());
    assert!(valtozasok.uj_mappak.is_empty());
    assert!(valtozasok.megszunt_mappak.is_empty());
    assert!(valtozasok.uj_kepek.is_empty());
    assert!(valtozasok.megszunt_kepek.is_empty());
    assert!(valtozasok.modosult_kepek.is_empty());
}
