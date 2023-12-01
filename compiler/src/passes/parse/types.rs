use derive_more::Display;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, Clone, Display)]
#[display(bound = "A: Display")]
pub enum Type<A> {
    #[display(fmt = "I64")]
    I64,
    #[display(fmt = "U64")]
    U64,
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
