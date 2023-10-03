use crate::language::alvar::Atom;
use crate::language::lvar::{Expr, LVarProgram, Op};

#[derive(Debug, PartialEq)]
pub struct CVarProgram {
    pub bdy: Tail,
}

#[derive(Debug, PartialEq)]
pub enum Tail {
    Return {
        expr: CExpr,
    },
    Seq {
        sym: String,
        bnd: CExpr,
        tail: Box<Tail>,
    },
}

#[derive(Debug, PartialEq)]
pub enum CExpr {
    Atom(Atom),
    Prim { op: Op, args: Vec<Atom> },
}

impl From<CVarProgram> for LVarProgram {
    fn from(value: CVarProgram) -> Self {
        LVarProgram {
            bdy: value.bdy.into(),
        }
    }
}

impl From<Tail> for Expr {
    fn from(value: Tail) -> Self {
        match value {
            Tail::Return { expr } => expr.into(),
            Tail::Seq { sym, bnd, tail } => Expr::Let {
                sym,
                bnd: Box::new(bnd.into()),
                bdy: Box::new((*tail).into()),
            },
        }
    }
}

impl From<CExpr> for Expr {
    fn from(value: CExpr) -> Self {
        match value {
            CExpr::Atom(a) => a.into(),
            CExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
        }
    }
}
