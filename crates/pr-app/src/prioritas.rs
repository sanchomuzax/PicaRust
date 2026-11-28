//! A háttérszálak alacsonyabb prioritása, ha az operációs rendszer ezt támogatja.

use thread_priority::{ThreadPriority, ThreadPriorityValue};

pub const HATTER: u8 = 20;
pub const LEGALACSONYABB: u8 = 1;

pub fn sajat_szal(szint: u8) {
    let prioritas = match ThreadPriorityValue::try_from(szint) {
        Ok(ertek) => ThreadPriority::Crossplatform(ertek),
        Err(hiba) => {
            crate::naplo(&format!("szálprioritás nem támogatott: {hiba}"));
            return;
        }
    };
    if let Err(hiba) = prioritas.set_for_current() {
        crate::naplo(&format!("szálprioritás beállítása sikertelen: {hiba}"));
    }
}

#[cfg(test)]
mod tesztek {
    use super::{HATTER, LEGALACSONYABB};

    #[test]
    fn az_alacsonyabb_szintek_platformfuggetlen_skalan_vannak() {
        assert_eq!(HATTER, 20);
        assert_eq!(LEGALACSONYABB, 1);
    }
}
