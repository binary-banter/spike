pub mod eliminate;
mod eliminate_params;
mod interpreter;
#[cfg(test)]
mod tests;
mod eliminate_tail;
mod eliminate_seq;
mod eliminate_expr;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Meta, Param, TypeDef, UnaryOp};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::select::io::Std;

pub struct PrgEliminated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, ETail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub enum ETail<'p> {
    Return {
        exprs: Vec<Atom<'p>>,
    },
    Seq {
        syms: Vec<UniqueSym<'p>>,
        bnd: Meta<Vec<Type<UniqueSym<'p>>>, EExpr<'p>>,
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
    },
    BinaryOp {
        op: BinaryOp,
        exprs: [Atom<'p>; 2],
    },
    UnaryOp {
        op: UnaryOp,
        expr: Atom<'p>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<Atom<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
}
