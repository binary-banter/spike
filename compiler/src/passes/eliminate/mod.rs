pub mod eliminate;
mod eliminate_params;
mod interpret;
#[cfg(test)]
mod tests;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{Op, Param, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgEliminated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, ETail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
}

pub enum ETail<'p> {
    Return {
        exprs: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
    },
    Seq {
        syms: Vec<UniqueSym<'p>>,
        bnd: EExpr<'p>,
        tail: Box<ETail<'p>>,
    },
    IfStmt {
        cnd: EExpr<'p>,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
}

pub enum EExpr<'p> {
    Atom {
        atm: Atom<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
        typs: Vec<Type<UniqueSym<'p>>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
}
