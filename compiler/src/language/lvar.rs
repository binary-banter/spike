use crate::interpreter::value::Val;
use std::collections::HashMap;

use crate::passes::type_check::Type;
use crate::passes::uniquify::UniqueSym;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct PrgParsed<'p> {
    pub defs: Vec<Def<&'p str>>,
    pub entry: &'p str,
}

pub type PrgTypeChecked<'p> = PrgGenericVar<&'p str>;
pub type PrgUniquified<'p> = PrgGenericVar<UniqueSym<'p>>;

#[derive(Debug, PartialEq)]
pub struct PrgGenericVar<A: Copy + Hash + Eq> {
    pub defs: HashMap<A, Def<A>>,
    pub entry: A,
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

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Read => "read",
                Op::Print => "print",
                Op::Plus => "plus",
                Op::Minus => "minus",
                Op::Mul => "multiply",
                Op::Div => "divide",
                Op::Mod => "modulo",
                Op::LAnd => "logical and",
                Op::LOr => "logical or",
                Op::Not => "not",
                Op::Xor => "xor",
                Op::GT => "greater",
                Op::GE => "greater or equal",
                Op::EQ => "equal",
                Op::LE => "less or equal",
                Op::LT => "less",
                Op::NE => "not equal",
            }
        )
    }
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Lit {
    Int { val: i64 },
    Bool { val: bool },
}

impl Lit {
    pub fn int(self) -> i64 {
        match self {
            Lit::Int { val } => val,
            Lit::Bool { .. } => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Lit::Int { .. } => panic!(),
            Lit::Bool { val } => val,
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
        }
    }
}

impl<A: Copy + Hash + Eq> From<Lit> for Val<A> {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => Val::Int { val },
            Lit::Bool { val } => Val::Bool { val },
        }
    }
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
}
