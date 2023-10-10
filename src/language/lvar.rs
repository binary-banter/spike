use crate::passes::uniquify::UniqueSym;
use std::fmt::{Display, Formatter};
use crate::language::x86var::Cnd;

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
    Greater,
    GreaterOrEqual,
    Equal,
    LessOrEqual,
    Less,
    NotEqual,
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
                Op::Greater => "greater",
                Op::GreaterOrEqual => "greater or equal",
                Op::Equal => "equal",
                Op::LessOrEqual => "less or equal",
                Op::Less => "less",
                Op::NotEqual => "not equal",
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr<A> {
    Int {
        val: i64,
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
        cnd: Cnd,
        thn: Box<Expr<A>>,
        els: Box<Expr<A>>,
    }
}
