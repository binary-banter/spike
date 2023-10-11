use crate::interpreter::value::Val;

use crate::passes::uniquify::UniqueSym;
use std::fmt::{Display, Formatter};

pub type LVarProgram<'p> = GLVarProgram<&'p str>;
pub type ULVarProgram<'p> = GLVarProgram<UniqueSym<'p>>;

#[derive(Debug, PartialEq)]
pub struct GLVarProgram<A> {
    pub bdy: Expr<A>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
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
}
