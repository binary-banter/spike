use crate::passes::atomize::{AExpr, Atom, DefAtomized, PrgAtomized};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, Typed};
use crate::passes::reveal::{DefRevealed, PrgRevealed, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

impl<'p> PrgRevealed<'p> {
    #[must_use]
    pub fn atomize(self) -> PrgAtomized<'p> {
        PrgAtomized {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, atomize_def(def)))
                .collect(),
            entry: self.entry,
            std: self.std,
        }
    }
}

fn atomize_def(def: DefRevealed) -> DefAtomized {
    match def {
        DefRevealed::Fn {
            sym,
            params,
            typ,
            bdy,
        } => {
            let mut scope =
                PushMap::from_iter(params.iter().map(|param| (param.sym, param.mutable)));

            DefAtomized::Fn {
                sym,
                params,
                typ,
                bdy: atomize_expr(bdy, &mut scope),
            }
        }
        DefRevealed::TypeDef { sym, def } => DefAtomized::TypeDef { sym, def },
    }
}

fn atomize_expr<'p>(
    expr: Typed<'p, RExpr<'p>>,
    scope: &mut PushMap<UniqueSym<'p>, bool>,
) -> Typed<'p, AExpr<'p>> {
    // Keep track of all the priors. These are bindings that should come before evaluating the expression.
    let mut priors = Vec::new();

    // Returns the atomized expression. This can hold `Var`s that must be bound by the priors.
    let inner = match expr.inner {
        RExpr::Lit { val } => AExpr::Atom {
            atm: Atom::Val { val },
        },
        RExpr::Var { sym } => AExpr::Atom {
            atm: Atom::Var { sym },
        },
        RExpr::BinaryOp {
            op,
            exprs: [lhs, rhs],
        } => AExpr::BinaryOp {
            op,
            exprs: [
                atomize_atom(*lhs, &mut priors, scope),
                atomize_atom(*rhs, &mut priors, scope),
            ],
        },
        RExpr::UnaryOp { op, expr: arg } => AExpr::UnaryOp {
            op,
            expr: atomize_atom(*arg, &mut priors, scope),
        },
        RExpr::Let {
            sym,
            mutable,
            bnd,
            bdy,
        } => {
            let bnd = Box::new(atomize_expr(*bnd, scope));
            let bdy = Box::new(scope.push(sym, mutable, |scope| atomize_expr(*bdy, scope)));

            AExpr::Let { sym, bnd, bdy }
        }
        RExpr::If { cnd, thn, els } => AExpr::If {
            cnd: Box::new(atomize_expr(*cnd, scope)),
            thn: Box::new(atomize_expr(*thn, scope)),
            els: Box::new(atomize_expr(*els, scope)),
        },
        RExpr::Apply { fun, args } => {
            let Type::Fn { params, .. } = fun.meta.clone() else {
                unreachable!()
            };

            AExpr::Apply {
                fun: atomize_atom(*fun, &mut priors, scope),
                args: args
                    .into_iter()
                    .map(|arg| atomize_atom(arg, &mut priors, scope))
                    .zip(params)
                    .collect(),
            }
        }
        RExpr::FunRef { sym } => AExpr::FunRef { sym },
        RExpr::Loop { bdy } => AExpr::Loop {
            bdy: Box::new(atomize_expr(*bdy, scope)),
        },
        RExpr::Break { bdy } => AExpr::Break {
            bdy: Box::new(atomize_expr(*bdy, scope)),
        },
        RExpr::Seq { stmt, cnt } => AExpr::Seq {
            stmt: Box::new(atomize_expr(*stmt, scope)),
            cnt: Box::new(atomize_expr(*cnt, scope)),
        },
        RExpr::Assign { sym, bnd } => AExpr::Assign {
            sym,
            bnd: Box::new(atomize_expr(*bnd, scope)),
        },
        RExpr::Continue => AExpr::Continue,
        RExpr::Return { bdy } => AExpr::Return {
            bdy: Box::new(atomize_expr(*bdy, scope)),
        },
        RExpr::Struct { sym, fields } => AExpr::Struct {
            sym,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| {
                    let field = atomize_atom(expr, &mut priors, scope);
                    (sym, field)
                })
                .collect(),
        },
        RExpr::AccessField { strct, field } => AExpr::AccessField {
            strct: atomize_atom(*strct, &mut priors, scope),
            field,
        },
        RExpr::Asm { instrs } => AExpr::Asm { instrs },
    };

    // Chains all the priors with the atomized expression as the body.
    let inner = priors
        .into_iter()
        .rfold(inner, |bdy, (sym, bnd)| AExpr::Let {
            sym,
            bnd: Box::new(bnd),
            bdy: Box::new(Meta {
                inner: bdy,
                meta: expr.meta.clone(),
            }),
        });

    Meta {
        inner,
        meta: expr.meta,
    }
}

fn atomize_atom<'p>(
    expr: Typed<'p, RExpr<'p>>,
    priors: &mut Vec<(UniqueSym<'p>, Typed<'p, AExpr<'p>>)>,
    scope: &mut PushMap<UniqueSym<'p>, bool>,
) -> Atom<'p> {
    match expr.inner {
        RExpr::Lit { val } => Atom::Val { val },
        RExpr::Var { sym } if !scope[&sym] => Atom::Var { sym },
        _ => {
            let tmp = gen_sym("tmp");
            priors.push((tmp, atomize_expr(expr, scope)));
            Atom::Var { sym: tmp }
        }
    }
}
