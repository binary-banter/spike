#[rustfmt::skip]
#[allow(clippy::all)]
mod grammar;
pub mod interpreter;

use crate::passes::parse::grammar::ProgramParser;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use crate::interpreter::value::Val;
use std::collections::HashMap;
use crate::passes::type_check::Type;
use crate::passes::uniquify::UniqueSym;
use std::fmt::{Display, Formatter};
use std::hash::Hash;

#[derive(Error, Debug, Diagnostic)]
#[error("Parse error!")]
#[diagnostic(
    code(oops::my::bad),
    url(docsrs),
    help("try doing it better next time?")
)]
pub struct PrettyParseError {
    #[source_code]
    src: String,

    #[label("Failed to parse here")]
    fail: SourceSpan,
}

pub fn parse_program(src: &str) -> Result<PrgParsed, PrettyParseError> {
    ProgramParser::new().parse(src).map_err(|e| {
        dbg!(e);
        panic!();
    })
}

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
        e: Option<Box<Expr<A>>>,
    }
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
    Unit
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
            Lit::Unit => write!(f, "unit")
        }
    }
}

impl<A: Copy + Hash + Eq> From<Lit> for Val<A> {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => Val::Int { val },
            Lit::Bool { val } => Val::Bool { val },
            Lit::Unit => Val::Unit
        }
    }
}
