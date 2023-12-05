pub mod conclude;

use crate::passes::assign::Arg;
use crate::passes::select::Block;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;


pub struct X86Concluded<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
}
