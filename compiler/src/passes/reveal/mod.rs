mod display;
pub mod reveal;

use crate::passes::parse::{BinaryOp, Def, Typed, UnaryOp};
use crate::passes::select::std_lib::Std;
use crate::passes::select::InstrSelected;
use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Display)]
#[display(fmt = "{}", r#"defs.values().format("\n")"#)]
pub struct PrgRevealed<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefRevealed<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub type DefRevealed<'p> = Def<UniqueSym<'p>, &'p str, Typed<'p, RExpr<'p>>>;

pub enum RExpr<'p> {
    Lit {
        val: TLit,
    },
    Var {
        sym: UniqueSym<'p>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    BinaryOp {
        op: BinaryOp,
        exprs: [Box<Typed<'p, RExpr<'p>>>; 2],
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Typed<'p, RExpr<'p>>>,
    },
    Let {
        sym: UniqueSym<'p>,
        mutable: bool,
        bnd: Box<Typed<'p, RExpr<'p>>>,
        bdy: Box<Typed<'p, RExpr<'p>>>,
    },
    If {
        cnd: Box<Typed<'p, RExpr<'p>>>,
        thn: Box<Typed<'p, RExpr<'p>>>,
        els: Box<Typed<'p, RExpr<'p>>>,
    },
    Apply {
        fun: Box<Typed<'p, RExpr<'p>>>,
        args: Vec<Typed<'p, RExpr<'p>>>,
    },
    Loop {
        bdy: Box<Typed<'p, RExpr<'p>>>,
    },
    Break {
        bdy: Box<Typed<'p, RExpr<'p>>>,
    },
    Return {
        bdy: Box<Typed<'p, RExpr<'p>>>,
    },
    Continue,
    Seq {
        stmt: Box<Typed<'p, RExpr<'p>>>,
        cnt: Box<Typed<'p, RExpr<'p>>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<Typed<'p, RExpr<'p>>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Typed<'p, RExpr<'p>>)>,
    },
    AccessField {
        strct: Box<Typed<'p, RExpr<'p>>>,
        field: &'p str,
    },
    Asm {
        instrs: Vec<InstrSelected<'p>>,
    },
}
