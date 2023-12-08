pub mod eliminate;
mod eliminate_expr;
mod eliminate_params;
mod eliminate_seq;
mod eliminate_tail;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Meta, Param, TypeDef, UnaryOp};
use crate::passes::select::InstrSelected;
use crate::utils::unique_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgEliminated<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunEliminated<'p>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
}

pub struct FunEliminated<'p> {
    pub params: Vec<Param<UniqueSym<'p>>>,
    pub blocks: HashMap<UniqueSym<'p>, TailEliminated<'p>>,
    pub entry: UniqueSym<'p>,
}

pub enum TailEliminated<'p> {
    Return {
        exprs: Vec<Atom<'p>>,
    },
    Seq {
        syms: Vec<UniqueSym<'p>>,
        bnd: Meta<Vec<Type<UniqueSym<'p>>>, ExprEliminated<'p>>,
        tail: Box<TailEliminated<'p>>,
    },
    IfStmt {
        cnd: ExprEliminated<'p>,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
}

pub enum ExprEliminated<'p> {
    Atom { atm: Atom<'p> },
    BinaryOp { op: BinaryOp, exprs: [Atom<'p>; 2] },
    UnaryOp { op: UnaryOp, expr: Atom<'p> },
    Apply { fun: Atom<'p>, args: Vec<Atom<'p>> },
    FunRef { sym: UniqueSym<'p> },
    Asm { instrs: Vec<InstrSelected<'p>> },
}
