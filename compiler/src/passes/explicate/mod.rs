mod display;
pub mod explicate;
mod explicate_assign;
mod explicate_pred;
mod explicate_tail;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Param, TypeDef, Typed, UnaryOp};
use crate::passes::select::InstrSelected;
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Display)]
#[display(
    fmt = "{}",
    r#"fns.iter().map(|(sym, fun)| format!("{sym}:\n{fun}")).format("\n")"#
)]
pub struct PrgExplicated<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunExplicated<'p>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Display)]
#[display(fmt = "")]
pub struct FunExplicated<'p> {
    pub params: Vec<Param<UniqueSym<'p>>>,
    pub blocks: HashMap<UniqueSym<'p>, TailExplicated<'p>>,
    pub entry: UniqueSym<'p>,
}

pub enum TailExplicated<'p> {
    Return {
        expr: Typed<'p, Atom<'p>>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: Typed<'p, ExprExplicated<'p>>,
        tail: Box<TailExplicated<'p>>,
    },
    IfStmt {
        cnd: ExprExplicated<'p>,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
}

pub enum ExprExplicated<'p> {
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
    Asm {
        instrs: Vec<InstrSelected<'p>>,
    },
}
