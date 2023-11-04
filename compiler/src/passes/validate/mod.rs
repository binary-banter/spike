mod type_check;
pub mod validate;
mod check_sized;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Lit, Op};
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use thiserror::Error;
use miette::Diagnostic;
use crate::passes::validate::type_check::error::TypeError;

#[derive(Debug, PartialEq)]
pub struct PrgTypeChecked<'p> {
    pub defs: HashMap<&'p str, Def<'p, &'p str, TExpr<'p, &'p str>>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub enum TExpr<'p, A: Copy + Hash + Eq + Display> {
    Lit {
        val: Lit,
        typ: Type<A>,
    },
    Var {
        sym: A,
        typ: Type<A>,
    },
    Prim {
        op: Op,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Let {
        sym: A,
        bnd: Box<TExpr<'p, A>>,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    If {
        cnd: Box<TExpr<'p, A>>,
        thn: Box<TExpr<'p, A>>,
        els: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Apply {
        fun: Box<TExpr<'p, A>>,
        args: Vec<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Loop {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Break {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Continue {
        typ: Type<A>,
    },
    Return {
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Seq {
        stmt: Box<TExpr<'p, A>>,
        cnt: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Assign {
        sym: A,
        bnd: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    Struct {
        sym: A,
        fields: Vec<(&'p str, TExpr<'p, A>)>,
        typ: Type<A>,
    },
    Variant {
        enum_sym: A,
        variant_sym: A,
        bdy: Box<TExpr<'p, A>>,
        typ: Type<A>,
    },
    AccessField {
        strct: Box<TExpr<'p, A>>,
        field: &'p str,
        typ: Type<A>,
    },
    Switch {
        enm: Box<TExpr<'p, A>>,
        arms: Vec<(A, A, Box<TExpr<'p, A>>)>,
        typ: Type<A>,
    },
}

impl<'p, A: Copy + Hash + Eq + Display> TExpr<'p, A> {
    pub fn typ(&self) -> &Type<A> {
        match self {
            TExpr::Lit { typ, .. }
            | TExpr::Var { typ, .. }
            | TExpr::Prim { typ, .. }
            | TExpr::Let { typ, .. }
            | TExpr::If { typ, .. }
            | TExpr::Apply { typ, .. }
            | TExpr::Loop { typ, .. }
            | TExpr::Break { typ, .. }
            | TExpr::Continue { typ, .. }
            | TExpr::Return { typ, .. }
            | TExpr::Seq { typ, .. }
            | TExpr::Assign { typ, .. }
            | TExpr::Struct { typ, .. }
            | TExpr::Variant { typ, .. }
            | TExpr::AccessField { typ, .. }
            | TExpr::Switch { typ, .. } => typ,
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum ValidateError {
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error("The program doesn't have a main function.")]
    NoMain,
    #[error("The type '{sym}' is not sized.")]
    UnsizedType { sym: String }
}
