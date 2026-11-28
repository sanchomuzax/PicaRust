//! A PicaRust magja. Ebben a mérföldkőben csak annyi van benne, hogy a CI-nak
//! legyen mit fordítania és tesztelnie.
#![forbid(unsafe_code)]

/// A csomag verzióját adja vissza (a `Cargo.toml`-ból).
pub fn verzio() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::verzio;

    #[test]
    fn verzio_az_aktualis_csomagverziot_adja() {
        assert_eq!(verzio(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn verzio_nem_ures() {
        assert!(!verzio().is_empty());
    }
}

#[cfg(test)]
mod kapuproba {
    #[test]
    fn a_kapuproba_tesztje_bukik() {
        assert_eq!(1 + 1, 3, "a kapupróba szándékos hibája");
    }
}
