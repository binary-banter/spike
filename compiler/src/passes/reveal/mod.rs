pub mod reveal;
#[cfg(test)]
mod tests;

use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def};
use crate::passes::validate::{ExprValidated, TLit};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgRevealed<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<UniqueSym<'p>, &'p str, RExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

pub enum RExpr<'p> {
    Lit {
        val: TLit,
        typ: Type<UniqueSym<'p>>,
    },
    Var {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: BinaryOp,
        args: Vec<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    If {
        cnd: Box<RExpr<'p>>,
        thn: Box<RExpr<'p>>,
        els: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Box<RExpr<'p>>,
        args: Vec<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Loop {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Break {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Return {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Continue {
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        stmt: Box<RExpr<'p>>,
        cnt: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, RExpr<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Box<RExpr<'p>>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
}

impl<'p> RExpr<'p> {
    pub fn typ(&self) -> &Type<UniqueSym<'p>> {
        match self {
            RExpr::Var { typ, .. }
            | RExpr::Lit { typ, .. }
            | RExpr::Prim { typ, .. }
            | RExpr::Let { typ, .. }
            | RExpr::If { typ, .. }
            | RExpr::Apply { typ, .. }
            | RExpr::Loop { typ, .. }
            | RExpr::Break { typ, .. }
            | RExpr::Continue { typ, .. }
            | RExpr::Return { typ, .. }
            | RExpr::Seq { typ, .. }
            | RExpr::Assign { typ, .. }
            | RExpr::Struct { typ, .. }
            | RExpr::AccessField { typ, .. }
            | RExpr::FunRef { typ, .. } => typ,
        }
    }
}

// impl<'p> From<PrgRevealed<'p>> for PrgUniquified<'p> {
//     fn from(value: PrgRevealed<'p>) -> Self {
//         PrgUniquified {
//             defs: value
//                 .defs
//                 .into_iter()
//                 .map(|(sym, def)| (sym, def.into()))
//                 .collect(),
//             entry: value.entry,
//         }
//     }
// }

// impl<'p> From<Def<UniqueSym<'p>, RExpr<'p>>>
//     for Def<UniqueSym<'p>, TExpr<'p>>
// {
//     fn from(value: Def<UniqueSym<'p>, RExpr<'p>>) -> Self {
//         match value {
//             Def::Fn {
//                 sym,
//                 params,
//                 typ,
//                 bdy,
//             } => Def::Fn {
//                 sym,
//                 params,
//                 typ,
//                 bdy: bdy.into(),
//             },
//             Def::TypeDef { sym, def } => Def::TypeDef { sym, def },
//         }
//     }
// }

// impl<'p> From<RExpr<'p>> for ExprValidated<'p> {
//     fn from(value: RExpr<'p>) -> Self {
//         match value {
//             RExpr::Lit { val, typ } => ExprValidated::Lit { val, typ },
//             RExpr::Prim { op, args, typ } => ExprValidated::Prim {
//                 op,
//                 args: args.into_iter().map(Into::into).collect(),
//                 typ,
//             },
//             RExpr::Let { sym, bnd, bdy, typ } => ExprValidated::Let {
//                 sym,
//                 bnd: Box::new((*bnd).into()),
//                 bdy: Box::new((*bdy).into()),
//                 typ,
//             },
//             RExpr::If { cnd, thn, els, typ } => ExprValidated::If {
//                 cnd: Box::new((*cnd).into()),
//                 thn: Box::new((*thn).into()),
//                 els: Box::new((*els).into()),
//                 typ,
//             },
//             RExpr::Apply { fun, args, typ } => ExprValidated::Apply {
//                 fun: Box::new((*fun).into()),
//                 args: args.into_iter().map(Into::into).collect(),
//                 typ,
//             },
//             RExpr::Var { sym, typ } | RExpr::FunRef { sym, typ } => ExprValidated::Var { sym, typ },
//             RExpr::Loop { bdy, typ } => ExprValidated::Loop {
//                 bdy: Box::new((*bdy).into()),
//                 typ,
//             },
//             RExpr::Break { bdy, typ } => ExprValidated::Break {
//                 bdy: Box::new((*bdy).into()),
//                 typ,
//             },
//             RExpr::Seq { stmt, cnt, typ } => ExprValidated::Seq {
//                 stmt: Box::new((*stmt).into()),
//                 cnt: Box::new((*cnt).into()),
//                 typ,
//             },
//             RExpr::Assign { sym, bnd, typ } => ExprValidated::Assign {
//                 sym,
//                 bnd: Box::new((*bnd).into()),
//                 typ,
//             },
//             RExpr::Continue { typ } => ExprValidated::Continue { typ },
//             RExpr::Return { bdy, typ } => ExprValidated::Return {
//                 bdy: Box::new((*bdy).into()),
//                 typ,
//             },
//             RExpr::Struct { sym, fields, typ } => ExprValidated::Struct {
//                 sym,
//                 fields: fields
//                     .into_iter()
//                     .map(|(sym, expr)| (sym, expr.into()))
//                     .collect(),
//                 typ,
//             },
//             RExpr::AccessField { strct, field, typ } => ExprValidated::AccessField {
//                 strct: Box::new((*strct).into()),
//                 field,
//                 typ,
//             },
//         }
//     }
// }
