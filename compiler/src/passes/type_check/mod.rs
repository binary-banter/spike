pub mod check;

use crate::passes::parse::PrgGenericVar;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::fmt::Display;
use std::hash::Hash;

pub type PrgTypeChecked<'p> = PrgGenericVar<&'p str>;

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

#[cfg(test)]
mod tests {
    use crate::passes::parse::parse::parse_program;
    use test_each_file::test_each_file;

    fn check([test]: [&str; 1], should_fail: bool) {
        let mut test = test.split('#');
        let program = test.nth(3).unwrap().trim();
        let program = parse_program(program).unwrap();
        let res = program.type_check();

        match (res, should_fail) {
            (Ok(_), true) => panic!("Program should not pass type-checking."),
            (Err(e), false) => {
                panic!("Program should have passed type-checking, but returned error: '{e}'.")
            }
            _ => {}
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as type_check_succeed => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/fail/type_check" as type_check_fail => |p| check(p, true) }
}
