use crate::passes::interference::Arg;
use crate::passes::select::io::Std;
use crate::passes::select::{Block, X86Selected};
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

pub mod conclude;

#[derive(Debug, PartialEq)]
pub struct X86Concluded<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

impl<'p> From<X86Concluded<'p>> for X86Selected<'p> {
    fn from(value: X86Concluded<'p>) -> Self {
        X86Selected {
            blocks: value.blocks.fmap(|v| v.fmap(Into::into)),
            entry: value.entry,
            std: value.std,
        }
    }
}