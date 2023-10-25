#[rustfmt::skip]
#[allow(clippy::all)]
mod grammar;
pub mod interpreter;
pub mod parse;

use crate::passes::type_check::Type;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Lit {
    Int { val: i64 },
    Bool { val: bool },
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

impl Display for Lit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Int { val } => write!(f, "{val}"),
            Lit::Bool { val } => {
                if *val {
                    write!(f, "true")
                } else {
                    write!(f, "false")
                }
            }
            Lit::Unit => write!(f, "unit"),
        }
    }
}
