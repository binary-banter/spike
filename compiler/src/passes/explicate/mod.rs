pub mod explicate;
mod explicate_assign;
mod explicate_def;
mod explicate_pred;
mod explicate_tail;
pub mod interpreter;
#[cfg(test)]
mod tests;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Param, Typed, TypeDef, UnaryOp};
use crate::passes::select::std_lib::Std;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, CTail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub enum CTail<'p> {
    Return {
        expr: Typed<'p, Atom<'p>>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: Typed<'p, CExpr<'p>>,
        tail: Box<CTail<'p>>,
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

pub enum CExpr<'p> {
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
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Struct {
        sym: UniqueSym<'p>,
        // todo: this does not need to be atom!
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
    },
}
