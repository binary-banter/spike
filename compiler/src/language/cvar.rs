use crate::language::alvar::Atom;
use crate::passes::parse::Op;
use crate::passes::uniquify::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Tail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<UniqueSym<'p>>>,
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
    Atom {
        atm: Atom<'p>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
    },
    Apply {
        fun: Box<Atom<'p>>,
        args: Vec<Atom<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
}
