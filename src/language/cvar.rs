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

impl Into<LVarProgram> for CVarProgram {
    fn into(self) -> LVarProgram {
        LVarProgram {
            bdy: self.bdy.into(),
        }
    }
}

impl Into<Expr> for Tail {
    fn into(self) -> Expr {
        match self {
            Tail::Return { expr } => expr.into(),
            Tail::Seq { sym, bnd, tail } => Expr::Let {
                sym,
                bnd: Box::new(bnd.into()),
                bdy: Box::new((*tail).into()),
            },
        }
    }
}

impl Into<Expr> for CExpr {
    fn into(self) -> Expr {
        match self {
            CExpr::Atom(a) => a.into(),
            CExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
        }
    }
}
