use crate::passes::parse::{Def, Expr, PrgParsed, Spanned};
use crate::passes::validate::error::TypeError;
use crate::utils::gen_sym::UniqueSym;

#[derive(Debug, PartialEq)]
pub struct PrgUniquified<'p> {
    /// The global program definitions.
    pub defs: Vec<Def<'p, &'p str, Spanned<Expr<'p, UniqueSym<'p>>>>>,
    /// The symbol representing the entry point of the program.
    pub entry: &'p str,
}

impl<'p> PrgParsed<'p> {
    pub fn uniquify(self) -> Result<PrgUniquified<'p>, TypeError> {
        todo!()
    }
}