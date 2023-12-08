use derive_more::Display;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd, Display)]
#[display(fmt = "{sym}.{id}")]
pub struct UniqueSym<'p> {
    pub sym: &'p str,
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

pub fn gen_sym(sym: &str) -> UniqueSym<'_> {
    UniqueSym {
        sym,
        id: COUNT.fetch_add(1, Ordering::SeqCst),
    }
}
