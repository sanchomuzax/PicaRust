//! Kétszintű munkasor: a látható képek (sürgős, a legutóbb kért elöl) megelőzik a háttérmunkát.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

struct Belso<T> {
    surgos: Vec<T>,
    hatter: VecDeque<T>,
}

pub struct Sorban<T> {
    belso: Mutex<Belso<T>>,
    jelzes: Condvar,
}

impl<T> Sorban<T> {
    pub fn uj() -> Self {
        Self {
            belso: Mutex::new(Belso {
                surgos: Vec::new(),
                hatter: VecDeque::new(),
            }),
            jelzes: Condvar::new(),
        }
    }

    pub fn surgos(&self, elem: T) {
        self.zar().surgos.push(elem);
        self.jelzes.notify_one();
    }

    pub fn hatter<I: IntoIterator<Item = T>>(&self, elemek: I) {
        self.zar().hatter.extend(elemek);
        self.jelzes.notify_all();
    }

    pub fn hossz(&self) -> usize {
        let b = self.zar();
        b.surgos.len() + b.hatter.len()
    }

    /// Vár, amíg van elem.
    pub fn kivesz(&self) -> T {
        let mut b = self.zar();
        loop {
            if let Some(elem) = b.surgos.pop() {
                return elem;
            }
            if let Some(elem) = b.hatter.pop_front() {
                return elem;
            }
            b = self.jelzes.wait(b).unwrap_or_else(PoisonError::into_inner);
        }
    }

    fn zar(&self) -> MutexGuard<'_, Belso<T>> {
        self.belso.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tesztek {
    use super::*;

    #[test]
    fn a_surgos_elozi_a_hatteret_es_forditott_sorrendu() {
        let s = Sorban::uj();
        s.hatter([1, 2]);
        s.surgos(3);
        s.surgos(4);
        assert_eq!(s.hossz(), 4);
        assert_eq!(
            [s.kivesz(), s.kivesz(), s.kivesz(), s.kivesz()],
            [4, 3, 1, 2]
        );
        assert_eq!(s.hossz(), 0);
    }
}
