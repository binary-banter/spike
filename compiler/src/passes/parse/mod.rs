#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic)]
mod grammar;
pub mod interpreter;
pub mod parse;

use crate::passes::type_check::Type;
use derive_more::Display;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct PrgParsed<'p> {
    pub defs: Vec<Def<&'p str, Expr<&'p str>>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub struct PrgGenericVar<A: Copy + Hash + Eq> {
    pub defs: HashMap<A, Def<A, Expr<A>>>,
    pub entry: A,
}

#[derive(Debug, PartialEq)]
pub enum Def<A: Copy + Hash + Eq, B> {
    Fn {
        sym: A,
        params: Vec<Param<A>>,
        typ: Type,
        bdy: B,
    },
}

#[derive(Debug, PartialEq)]
pub struct Param<A: Copy + Hash + Eq> {
    pub sym: A,
    pub typ: Type,
    pub mutable: bool,
}

#[derive(Debug, PartialEq)]
pub enum Expr<A: Copy + Hash + Eq> {
    Lit {
        val: Lit,
    },
    Var {
        sym: A,
    },
    Prim {
        op: Op,
        args: Vec<Expr<A>>,
    },
    Let {
        sym: A,
        mutable: bool,
        bnd: Box<Expr<A>>,
        bdy: Box<Expr<A>>,
    },
    If {
        cnd: Box<Expr<A>>,
        thn: Box<Expr<A>>,
        els: Box<Expr<A>>,
    },
    Apply {
        fun: Box<Expr<A>>,
        args: Vec<Expr<A>>,
    },
    Loop {
        bdy: Box<Expr<A>>,
    },
    Break {
        bdy: Box<Expr<A>>,
    },
    Continue,
    Seq {
        stmt: Box<Expr<A>>,
        cnt: Box<Expr<A>>,
    },
    Assign {
        sym: A,
        bnd: Box<Expr<A>>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    LAnd,
    LOr,
    Not,
    Xor,
    GT,
    GE,
    EQ,
    LE,
    LT,
    NE,
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum Lit {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
}

impl Lit {
    #[must_use]
    pub fn int(self) -> i64 {
        if let Lit::Int { val } = self {
            val
        } else {
            panic!()
        }
    }

    #[must_use]
    pub fn bool(self) -> bool {
        if let Lit::Bool { val } = self {
            val
        } else {
            panic!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn parse([test]: [&str; 1]) {
        let _ = split_test(test);
    }

    test_each_file! { for ["test"] in "./programs/good" as parse => parse }
}
