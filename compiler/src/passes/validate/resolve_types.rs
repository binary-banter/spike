use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{PrgConstrained, PrgValidated};

impl<'p> PrgConstrained<'p> {
    #[must_use]
    pub fn resolve(self) -> PrgValidated<'p> {
        PrgValidated {
            defs: Default::default(),
            entry: self.entry,
        }
    }
}
