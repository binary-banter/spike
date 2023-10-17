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
pub struct PrgGenericVar<A: Hash + Eq + PartialEq> {
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
pub enum Def<A> {
    Fn {
        sym: A,
        args: Vec<(A, Type)>,
        typ: Type,
        bdy: Expr<A>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Expr<A> {
    Val {
        val: Val,
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
        sym: A,
        args: Vec<Expr<A>>,
    },
}
