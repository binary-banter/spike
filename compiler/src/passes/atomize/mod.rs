pub mod atomize;
#[cfg(test)]
mod tests;

use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Meta, UnaryOp};
use crate::passes::select::std_lib::Std;
use crate::passes::validate::{DefValidated, ExprValidated, PrgValidated, TLit};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefAtomized<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub type DefAtomized<'p> = Def<UniqueSym<'p>, &'p str, Meta<Type<UniqueSym<'p>>, AExpr<'p>>>;

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
        bnd: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
        bdy: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    If {
        cnd: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
        thn: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
        els: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Loop {
        bdy: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Break {
        bdy: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Continue,
    Seq {
        stmt: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
        cnt: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Return {
        bdy: Box<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
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

impl<'p> From<PrgAtomized<'p>> for PrgValidated<'p> {
    fn from(value: PrgAtomized<'p>) -> Self {
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

impl<'p> From<DefAtomized<'p>> for DefValidated<'p> {
    fn from(value: DefAtomized<'p>) -> Self {
        match value {
            DefAtomized::Fn {
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
            DefAtomized::TypeDef { sym, def } => DefValidated::TypeDef { sym, def },
        }
    }
}

impl<'p> From<Meta<Type<UniqueSym<'p>>, AExpr<'p>>>
    for Meta<Type<UniqueSym<'p>>, ExprValidated<'p>>
{
    fn from(value: Meta<Type<UniqueSym<'p>>, AExpr<'p>>) -> Self {
        let inner = match value.inner {
            AExpr::Atom { atm, .. } => return atm.into(),
            AExpr::BinaryOp { op, exprs } => ExprValidated::BinaryOp {
                op,
                exprs: exprs.map(|a| Box::new(a.into())),
            },
            AExpr::UnaryOp { op, expr } => ExprValidated::UnaryOp {
                op,
                expr: Box::new(expr.into()),
            },
            AExpr::Let { sym, bnd, bdy } => ExprValidated::Let {
                sym,
                mutable: true,
                typ: None,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
            AExpr::If { cnd, thn, els } => ExprValidated::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            AExpr::Apply { fun, args, .. } => ExprValidated::Apply {
                fun: Box::new(fun.into()),
                args: args.into_iter().map(|(a, _)| a.into()).collect(),
            },
            AExpr::FunRef { sym } => ExprValidated::Var { sym },
            AExpr::Loop { bdy } => ExprValidated::Loop {
                bdy: Box::new((*bdy).into()),
            },
            AExpr::Break { bdy } => ExprValidated::Break {
                bdy: Box::new((*bdy).into()),
            },
            AExpr::Seq { stmt, cnt } => ExprValidated::Seq {
                stmt: Box::new((*stmt).into()),
                cnt: Box::new((*cnt).into()),
            },
            AExpr::Assign { sym, bnd } => ExprValidated::Assign {
                sym,
                bnd: Box::new((*bnd).into()),
            },
            AExpr::Continue => ExprValidated::Continue,
            AExpr::Return { bdy } => ExprValidated::Return {
                bdy: Box::new((*bdy).into()),
            },
            AExpr::Struct { sym, fields } => ExprValidated::Struct {
                sym,
                fields: fields
                    .into_iter()
                    .map(|(sym, atm)| (sym, atm.into()))
                    .collect(),
            },
            AExpr::AccessField { strct, field } => ExprValidated::AccessField {
                strct: Box::new(strct.into()),
                field,
            },
        };

        Meta {
            inner,
            meta: value.meta,
        }
    }
}

// Note that casting to Never here is safe because this `From` is only used by the interpreter which doesn't care about the type information.
impl<'p> From<Atom<'p>> for Meta<Type<UniqueSym<'p>>, ExprValidated<'p>> {
    fn from(value: Atom<'p>) -> Self {
        Meta {
            meta: Type::Never,
            inner: match value {
                Atom::Val { val } => ExprValidated::Lit { val },
                Atom::Var { sym } => ExprValidated::Var { sym },
            },
        }
    }
}
