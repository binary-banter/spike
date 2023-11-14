pub mod explicate;
pub mod interpret;
#[cfg(test)]
mod tests;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Param, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, CTail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
}

pub enum CTail<'p> {
    Return {
        expr: Atom<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: CExpr<'p>,
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
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: BinaryOp,
        args: Vec<Atom<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
        typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        // todo: this does not need to be atom!
        fields: Vec<(&'p str, Atom<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
}

impl<'p> CExpr<'p> {
    pub fn typ(&self) -> &Type<UniqueSym<'p>> {
        match self {
            CExpr::Atom { typ, .. }
            | CExpr::Prim { typ, .. }
            | CExpr::Apply { typ, .. }
            | CExpr::FunRef { typ, .. }
            | CExpr::Struct { typ, .. }
            | CExpr::AccessField { typ, .. } => typ,
        }
    }
}
