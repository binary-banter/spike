pub mod atomize;

use crate::passes::parse::{Def, Expr, Lit, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<UniqueSym<'p>, AExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum AExpr<'p> {
    Atom {
        atm: Atom<'p>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
        bdy: Box<AExpr<'p>>,
    },
    If {
        cnd: Box<AExpr<'p>>,
        thn: Box<AExpr<'p>>,
        els: Box<AExpr<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<Atom<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Loop {
        bdy: Box<AExpr<'p>>,
    },
    Break {
        bdy: Atom<'p>,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Atom<'p> {
    Val { val: Lit },
    Var { sym: UniqueSym<'p> },
}

impl<'p> From<PrgAtomized<'p>> for PrgUniquified<'p> {
    fn from(value: PrgAtomized<'p>) -> Self {
        PrgUniquified {
            defs: value
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, def.into()))
                .collect(),
            entry: value.entry,
        }
    }
}

// TODO functor time
impl<'p> From<Def<UniqueSym<'p>, AExpr<'p>>> for Def<UniqueSym<'p>, Expr<UniqueSym<'p>>> {
    fn from(value: Def<UniqueSym<'p>, AExpr<'p>>) -> Self {
        match value {
            Def::Fn {
                sym,
                params,
                typ,
                bdy,
            } => Def::Fn {
                sym,
                params,
                typ,
                bdy: bdy.into(),
            },
        }
    }
}

impl<'p> From<AExpr<'p>> for Expr<UniqueSym<'p>> {
    fn from(value: AExpr<'p>) -> Self {
        match value {
            AExpr::Atom { atm } => atm.into(),
            AExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
            AExpr::Let { sym, bnd, bdy } => Expr::Let {
                sym,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
            AExpr::If { cnd, thn, els } => Expr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            AExpr::Apply { fun, args } => Expr::Apply {
                fun: Box::new(fun.into()),
                args: args.into_iter().map(Into::into).collect(),
            },
            AExpr::FunRef { sym } => Expr::Var { sym },
            AExpr::Loop { bdy } => Expr::Loop { bdy: Box::new((*bdy).into()) },
            AExpr::Break { bdy } => Expr::Break { bdy: Some(Box::new(bdy.into())) },
        }
    }
}

impl<'p> From<Atom<'p>> for Expr<UniqueSym<'p>> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Val { val } => Expr::Lit { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
