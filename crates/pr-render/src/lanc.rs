#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lepes {
    pub nev: String,
    pub parameterek: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ertelmezett {
    pub lepesek: Vec<Lepes>,
    pub elvagva: Option<String>,
}

pub fn ertelmez(lanc: &str) -> Ertelmezett {
    let mut lepesek = Vec::new();
    for (index, tag) in lanc.split(';').enumerate() {
        if tag.is_empty() {
            continue;
        }
        let Some((lanc_nev, nyers_mezok)) = tag.split_once('=') else {
            return elvagja(lepesek, index, tag, "hiányzik az egyenlőségjel");
        };
        if lanc_nev.is_empty() {
            return elvagja(lepesek, index, tag, "üres az effekt neve");
        }
        let Some((nev, vart_parameterek)) = crate::effekt_nev_es_parameterek(lanc_nev) else {
            return elvagja(
                lepesek,
                index,
                tag,
                "ismeretlen vagy nem kanonikus effekt név",
            );
        };

        let mezok: Vec<&str> = if nyers_mezok.is_empty() {
            Vec::new()
        } else {
            nyers_mezok.split(',').collect()
        };
        let parameter_mezok = if mezok.len() > 1 { &mezok[1..] } else { &[] };
        let parameterek: Vec<String> = parameter_mezok
            .iter()
            .map(|ertek| (*ertek).to_owned())
            .collect();
        let erdemi_parameterek = parameterek
            .iter()
            .rposition(|ertek| !ertek.is_empty())
            .map_or(0, |utolso| utolso + 1);
        if erdemi_parameterek > 0 && erdemi_parameterek != vart_parameterek {
            return elvagja(lepesek, index, tag, "hibás az effektparaméterek száma");
        }

        lepesek.push(Lepes {
            nev: nev.to_owned(),
            parameterek,
        });
    }
    Ertelmezett {
        lepesek,
        elvagva: None,
    }
}

fn elvagja(lepesek: Vec<Lepes>, index: usize, tag: &str, ok: &str) -> Ertelmezett {
    Ertelmezett {
        lepesek,
        elvagva: Some(format!("{}. tag (`{tag}`): {ok}", index + 1)),
    }
}

#[cfg(test)]
mod tesztek {
    use super::{ertelmez, Lepes};

    fn lepes(nev: &str, parameterek: &[&str]) -> Lepes {
        Lepes {
            nev: nev.to_owned(),
            parameterek: parameterek
                .iter()
                .map(|ertek| (*ertek).to_owned())
                .collect(),
        }
    }

    #[test]
    fn ures_lanc_ures_eredmenyt_ad() {
        assert_eq!(
            ertelmez(""),
            super::Ertelmezett {
                lepesek: Vec::new(),
                elvagva: None,
            }
        );
    }

    #[test]
    fn tobb_lepes_sorrendben_marad() {
        let eredmeny = ertelmez("Boost=1,75;Invert=1");

        assert_eq!(
            eredmeny.lepesek,
            vec![lepes("boost", &["75"]), lepes("invert", &[])]
        );
        assert_eq!(eredmeny.elvagva, None);
    }

    #[test]
    fn ismeretlen_nevnel_elvag_es_megtartja_az_elozot() {
        let eredmeny = ertelmez("Boost=1;NincsIlyen=1;Invert=1;");

        assert_eq!(eredmeny.lepesek, vec![lepes("boost", &[])]);
        let ok = eredmeny.elvagva.as_deref();
        assert!(ok.is_some_and(|ertek| ertek.contains("2") && ertek.contains("NincsIlyen")));
    }

    #[test]
    fn ures_tagot_atlep() {
        let eredmeny = ertelmez("Boost=1;;Invert=1;");

        assert_eq!(
            eredmeny.lepesek,
            vec![lepes("boost", &[]), lepes("invert", &[])]
        );
        assert_eq!(eredmeny.elvagva, None);
    }

    #[test]
    fn zaropontosvesszo_nelkul_ervenyes() {
        let eredmeny = ertelmez("Boost=1,75;Invert=1");

        assert_eq!(
            eredmeny,
            super::Ertelmezett {
                lepesek: vec![lepes("boost", &["75"]), lepes("invert", &[])],
                elvagva: None,
            }
        );
    }

    #[test]
    fn zaro_ures_parametert_elfogadja() {
        let eredmeny = ertelmez("Invert=1,;");

        assert_eq!(eredmeny.lepesek, vec![lepes("invert", &[""])]);
        assert_eq!(eredmeny.elvagva, None);
    }

    #[test]
    fn szokozoket_nem_vagja_le() {
        let eredmeny = ertelmez("Boost=1; Invert=1;Invert=1;");

        assert_eq!(eredmeny.lepesek, vec![lepes("boost", &[])]);
        let ok = eredmeny.elvagva.as_deref();
        assert!(ok.is_some_and(|ertek| ertek.contains(" Invert")));
    }

    #[test]
    fn nevillesztes_bajtpontos_es_a_flaget_nem_vizsgalja() {
        let eredmeny = ertelmez("Boost=0,75;boost=1;Invert=1;");

        assert_eq!(eredmeny.lepesek, vec![lepes("boost", &["75"])]);
        let ok = eredmeny.elvagva.as_deref();
        assert!(ok.is_some_and(|ertek| ertek.contains("boost")));
    }

    #[test]
    fn ures_nevnel_elvag() {
        let eredmeny = ertelmez("=1;Invert=1;");

        assert!(eredmeny.lepesek.is_empty());
        assert!(eredmeny.elvagva.is_some());
    }

    #[test]
    fn hibas_alaknal_elvag() {
        let eredmeny = ertelmez("Boost;Invert=1;");

        assert!(eredmeny.lepesek.is_empty());
        assert!(eredmeny.elvagva.is_some());
    }

    #[test]
    fn hibas_aritynal_elvag_es_nem_veszi_fel_a_lepest() {
        let eredmeny = ertelmez("Boost=1,75,12;Invert=1;");

        assert!(eredmeny.lepesek.is_empty());
        assert!(eredmeny.elvagva.is_some());
    }
}
