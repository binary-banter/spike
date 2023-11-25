pub mod reveal;
#[cfg(test)]
mod tests;

use crate::passes::parse::{BinaryOp, Def, Meta, Typed, UnaryOp};
use crate::passes::select::std_lib::Std;
use crate::passes::validate::{DefValidated, ExprValidated, PrgValidated, TLit};
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

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
}

impl<'p> From<PrgRevealed<'p>> for PrgValidated<'p> {
    fn from(value: PrgRevealed<'p>) -> Self {
        PrgValidated {
            defs: value
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, def.into()))
                .collect(),
            entry: value.entry,
            std: value.std,
        }
    }
}

impl<'p> From<DefRevealed<'p>> for DefValidated<'p> {
    fn from(value: DefRevealed<'p>) -> Self {
        match value {
            DefRevealed::Fn {
                sym,
                params,
                typ,
                bdy,
            } => DefValidated::Fn {
                sym,
                params,
                typ,
                bdy: bdy.into(),
            },
            DefRevealed::TypeDef { sym, def } => DefValidated::TypeDef { sym, def },
        }
    }
}

impl<'p> From<Typed<'p, RExpr<'p>>> for Typed<'p, ExprValidated<'p>> {
    fn from(value: Typed<'p, RExpr<'p>>) -> Self {
        let inner = match value.inner {
            RExpr::Lit { val } => ExprValidated::Lit { val },
            RExpr::BinaryOp { op, exprs } => ExprValidated::BinaryOp {
                op,
                exprs: exprs.map(|expr| expr.fmap(Into::into)),
            },
            RExpr::UnaryOp { op, expr } => ExprValidated::UnaryOp {
                op,
                expr: expr.fmap(Into::into),
            },
            RExpr::Let { sym, bnd, bdy } => ExprValidated::Let {
                sym,
                mutable: true,
                typ: None,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
            RExpr::If { cnd, thn, els } => ExprValidated::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            RExpr::Apply { fun, args } => ExprValidated::Apply {
                fun: Box::new((*fun).into()),
                args: args.into_iter().map(Into::into).collect(),
            },
            RExpr::Var { sym } | RExpr::FunRef { sym } => ExprValidated::Var { sym },
            RExpr::Loop { bdy } => ExprValidated::Loop {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Break { bdy } => ExprValidated::Break {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Seq { stmt, cnt } => ExprValidated::Seq {
                stmt: Box::new((*stmt).into()),
                cnt: Box::new((*cnt).into()),
            },
            RExpr::Assign { sym, bnd } => ExprValidated::Assign {
                sym,
                bnd: Box::new((*bnd).into()),
            },
            RExpr::Continue => ExprValidated::Continue,
            RExpr::Return { bdy } => ExprValidated::Return {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Struct { sym, fields } => ExprValidated::Struct {
                sym,
                fields: fields
                    .into_iter()
                    .map(|(sym, expr)| (sym, expr.into()))
                    .collect(),
            },
            RExpr::AccessField { strct, field } => ExprValidated::AccessField {
                strct: Box::new((*strct).into()),
                field,
            },
        };

        Meta {
            inner,
            meta: value.meta,
        }
    }
}
