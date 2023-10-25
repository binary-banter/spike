pub mod check;

use crate::passes::parse::PrgGenericVar;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

pub type PrgTypeChecked<'p> = PrgGenericVar<&'p str>;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Unit,
    Never,
    Fn { typ: Box<Type>, args: Vec<Type> },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Unit => write!(f, "Unit"),
            Type::Never => write!(f, "Never"),
            Type::Fn { typ, args } => write!(f, "fn({}) -> {}", args.iter().format(", "), typ),
        }
    }
}
