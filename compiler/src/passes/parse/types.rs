use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::fmt::Display;

#[derive(Debug, Clone, Display)]
pub enum Type<A: Display> {
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

impl<A: Display> Type<A> {
    pub fn fmap<__B: Display>(self, __f: impl Fn(A) -> __B) -> Type<__B> {
        fn fmap_ref<A: Display, __B: Display>(s: Type<A>, __f: &impl Fn(A) -> __B) -> Type<__B> {
            match s {
                Type::Int => Type::Int,
                Type::Bool => Type::Bool,
                Type::Unit => Type::Unit,
                Type::Never => Type::Never,
                Type::Fn { typ, params: args } => Type::Fn {
                    typ: typ.fmap(|v| fmap_ref(v, __f)),
                    params: args.fmap(|v| fmap_ref(v, __f)),
                },
                Type::Var { sym } => Type::Var { sym: __f(sym) },
            }
        }

        fmap_ref(self, &__f)
    }
}
