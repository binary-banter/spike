use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use derive_more::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct UniqueSym<'p> {
    #[cfg(release)]
    pub sym: &'p str,
    #[cfg(not(release))]
    pub sym: PhantomData<&'p str>,
    pub id: usize,
}

impl<'p> UniqueSym<'p> {
    pub fn fresh(self) -> Self {
        Self {
            sym: self.sym,
            id: COUNT.fetch_add(1, Ordering::SeqCst),
        }
    }
}

impl Display for UniqueSym<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(release)]
        let r = write!(f, "{}{}", self.sym, self.id);
        #[cfg(not(release))]
        let r = write!(f, "v{}", self.id);

        r
    }
}


#[cfg(not(release))]
pub fn gen_sym(_sym: &str) -> UniqueSym<'_> {
    UniqueSym {
        sym: PhantomData,
        id: COUNT.fetch_add(1, Ordering::SeqCst),
    }
}

#[cfg(release)]
pub fn gen_sym(sym: &str) -> UniqueSym<'_> {
    UniqueSym {
        sym,
        id: COUNT.fetch_add(1, Ordering::SeqCst),
    }
}
