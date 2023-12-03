use crate::passes::assign::Arg;
use crate::passes::select::Block;
use crate::utils::gen_sym::UniqueSym;

use std::collections::HashMap;

pub mod conclude;
#[cfg(test)]
mod tests;

pub struct X86Concluded<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
}
