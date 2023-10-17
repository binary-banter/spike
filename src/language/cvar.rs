use crate::language::alvar::Atom;
use crate::language::lvar::{Expr, Op};
use crate::passes::uniquify::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Tail<'p>>,
    pub entry: UniqueSym<'p>,
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
    IfStmt {
        cnd: CExpr<'p>,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CExpr<'p> {
    Atom(Atom<'p>),
    Prim { op: Op, args: Vec<Atom<'p>> },
}

impl<'p> From<CExpr<'p>> for Expr<UniqueSym<'p>> {
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
