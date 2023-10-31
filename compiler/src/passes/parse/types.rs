use std::hash::Hash;
use std::fmt::Display;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Type<A: Hash + Eq + Display> {
    #[display(fmt = "Int")]
    Int,
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

impl<A: Hash + Eq + Display> Type<A> {
    pub fn fmap<__B: Hash + Eq + Display>(self, __f: impl Fn(A) -> __B + Copy) -> Type<__B> {
        match self {
            Self::Int => Type::Int,
            Self::Bool => Type::Bool,
            Self::Unit => Type::Unit,
            Self::Never => Type::Never,
            Self::Fn { typ, params: args } => Type::Fn {
                typ: typ.fmap(|v| v.fmap(__f)),
                params: args.fmap(|v| v.fmap(__f)),
            },
            Self::Var { sym } => Type::Var { sym: __f(sym) },
        }
    }
}
