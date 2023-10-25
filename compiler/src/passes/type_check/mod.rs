pub mod check;

use crate::passes::parse::PrgGenericVar;
use derive_more::Display;
use itertools::Itertools;

pub type PrgTypeChecked<'p> = PrgGenericVar<&'p str>;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type {
    #[display(fmt = "Int")]
    Int,
    #[display(fmt = "Bool")]
    Bool,
    #[display(fmt = "Unit")]
    Unit,
    #[display(fmt = "Never")]
    Never,
    #[display(fmt = "fn({}) -> {typ}", r#"args.iter().format(", ")"#)]
    Fn { typ: Box<Type>, args: Vec<Type> },
}
