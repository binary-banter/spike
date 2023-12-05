pub mod patch;

use crate::passes::assign::FunAssigned;
use crate::passes::select::X86Selected;
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

pub struct X86Patched<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunAssigned<'p>>,
    pub entry: UniqueSym<'p>,
}

impl<'p> From<X86Patched<'p>> for X86Selected<'p> {
    fn from(value: X86Patched<'p>) -> Self {
        X86Selected {
            fns: value.fns.fmap(Into::into),
            entry: value.entry,
        }
    }
}
