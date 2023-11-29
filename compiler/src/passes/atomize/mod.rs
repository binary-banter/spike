pub mod atomize;

use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Meta, Typed, UnaryOp};
use crate::passes::select::std_lib::Std;
use crate::passes::validate::{DefValidated, ExprValidated, PrgValidated, TLit};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::select::{Instr, VarArg};

pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefAtomized<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub type DefAtomized<'p> = Def<UniqueSym<'p>, &'p str, Typed<'p, AExpr<'p>>>;

pub enum AExpr<'p> {
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
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<Typed<'p, AExpr<'p>>>,
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    If {
        cnd: Box<Typed<'p, AExpr<'p>>>,
        thn: Box<Typed<'p, AExpr<'p>>>,
        els: Box<Typed<'p, AExpr<'p>>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Loop {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Break {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Continue,
    Seq {
        stmt: Box<Typed<'p, AExpr<'p>>>,
        cnt: Box<Typed<'p, AExpr<'p>>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<Typed<'p, AExpr<'p>>>,
    },
    Return {
        bdy: Box<Typed<'p, AExpr<'p>>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
    },
    Asm {
        instrs: Vec<Instr<VarArg<UniqueSym<'p>>, UniqueSym<'p>>>,
    },
}

#[derive(Copy, Clone)]
pub enum Atom<'p> {
    Val { val: TLit },
    Var { sym: UniqueSym<'p> },
}

impl<'p> Atom<'p> {
    pub fn var(self) -> UniqueSym<'p> {
        if let Atom::Var { sym } = self {
            sym
        } else {
            panic!()
        }
    }
}
