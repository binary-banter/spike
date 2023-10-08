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
}
