pub mod check;
pub mod error;
mod uncover_globals;
mod util;
mod validate_expr;
mod validate_prim;
mod validate_struct;
mod validate_type;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Lit, Op};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct PrgTypeChecked<'p> {
    pub defs: HashMap<&'p str, Def<'p, &'p str, TExpr<'p, &'p str>>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub enum TExpr<'p, A: Copy + Hash + Eq + Display> {
    Lit {
        val: Lit,
        typ: Type<A>,
    },
    Var {
        sym: A,
        typ: Type<A>,
    },
    Prim {
        op: Op,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Let {
        sym: A,
        mutable: bool,
        bnd: Box<TExpr<'p, A>>,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    If {
        cnd: Box<TExpr<'p, A>>,
        thn: Box<TExpr<'p, A>>,
        els: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Apply {
        fun: Box<TExpr<'p, A>>,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Loop {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Break {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Continue {
        typ: Type<A>,
    },
    Return {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Seq {
        stmt: Box<TExpr<'p, A>>,
        cnt: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Assign {
        sym: A,
        bnd: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Struct {
        sym: A,
        fields: Vec<(&'p str, TExpr<'p, A>)>,
        typ: Type<A>,
    },
    Variant {
        enum_sym: A,
        variant_sym: A,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    AccessField {
        strct: Box<TExpr<'p, A>>,
        field: &'p str,
        typ: Type<A>,
    },
    Switch {
        enm: Box<TExpr<'p, A>>,
        arms: Vec<(A, A, Box<TExpr<'p, A>>)>,
        typ: Type<A>,
    },
}

impl<'p, A: Copy + Hash + Eq + Display> TExpr<'p, A> {
    pub fn typ(&self) -> &Type<A> {
        match self {
            TExpr::Lit { typ, .. }
            | TExpr::Var { typ, .. }
            | TExpr::Prim { typ, .. }
            | TExpr::Let { typ, .. }
            | TExpr::If { typ, .. }
            | TExpr::Apply { typ, .. }
            | TExpr::Loop { typ, .. }
            | TExpr::Break { typ, .. }
            | TExpr::Continue { typ, .. }
            | TExpr::Return { typ, .. }
            | TExpr::Seq { typ, .. }
            | TExpr::Assign { typ, .. }
            | TExpr::Struct { typ, .. }
            | TExpr::Variant { typ, .. }
            | TExpr::AccessField { typ, .. }
            | TExpr::Switch { typ, .. } => typ,
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
