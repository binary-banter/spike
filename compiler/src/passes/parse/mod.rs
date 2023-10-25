#[rustfmt::skip]
#[allow(clippy::all)]
mod grammar;
pub mod interpreter;
pub mod parse;

use crate::passes::type_check::Type;
use derive_more::Display;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct PrgParsed<'p> {
    pub defs: Vec<Def<&'p str>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub struct PrgGenericVar<A: Copy + Hash + Eq> {
    pub defs: HashMap<A, Def<A>>,
    pub entry: A,
}

#[derive(Debug, PartialEq)]
pub enum Def<A: Copy + Hash + Eq> {
    Fn {
        sym: A,
        params: Vec<(A, Type)>,
        typ: Type,
        bdy: Expr<A>,
    },
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
        bdy: Option<Box<Expr<A>>>,
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
    pub fn int(self) -> i64 {
        match self {
            Lit::Int { val } => val,
            _ => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Lit::Bool { val } => val,
            _ => panic!(),
        }
    }
}
