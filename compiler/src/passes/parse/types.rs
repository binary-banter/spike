use derive_more::Display;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, Clone, Display)]
#[display(bound = "A: Display")]
pub enum Type<A> {
    #[display(fmt = "{_0}")]
    Int(IntType),
    #[display(fmt = "Bool")]
    Bool,
    #[display(fmt = "Unit")]
    Unit,
    #[display(fmt = "Never")]
    Never,
    #[display(fmt = "fn({}) -> {typ}", r#"params.iter().format(", ")"#)]
    Fn {
        params: Vec<Type<A>>,
        typ: Box<Type<A>>,
    },
    #[display(fmt = "{sym}")]
    Var { sym: A },
}

/// Integer types
#[derive(Display, Debug, Copy, Clone, Eq, PartialEq)]
pub enum IntType {
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}
