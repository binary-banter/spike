use crate::language::alvar::Atom;
use crate::language::lvar::{Expr, LVarProgram, Op, ULVarProgram};
use crate::passes::uniquify::UniqueSym;

#[derive(Debug, PartialEq)]
pub struct CVarProgram<'p> {
    pub bdy: Tail<'p>,
}

#[derive(Debug, PartialEq)]
pub enum Tail<'p> {
    Return {
        expr: CExpr<'p>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: CExpr<'p>,
        tail: Box<Tail<'p>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum CExpr<'p> {
    Atom(Atom<'p>),
    Prim { op: Op, args: Vec<Atom<'p>> },
}

impl<'p> From<CVarProgram<'p>> for ULVarProgram<'p> {
    fn from(value: CVarProgram<'p>) -> Self {
        ULVarProgram {
            bdy: value.bdy.into(),
        }
    }
}

impl<'p> From<Tail<'p>> for Expr< UniqueSym<'p>> {
    fn from(value: Tail<'p>) -> Self {
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

impl<'p> From<CExpr<'p>> for Expr< UniqueSym<'p>> {
    fn from(value: CExpr<'p>) -> Self {
        match value {
            CExpr::Atom(a) => a.into(),
            CExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
        }
    }
}
