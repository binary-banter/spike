use crate::passes::atomize::{AExpr, Atom, DefAtomized, PrgAtomized};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, Typed};
use crate::passes::reveal::{DefRevealed, PrgRevealed, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};

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
        } => DefAtomized::Fn {
            sym,
            params,
            typ,
            bdy: atomize_expr(bdy),
        },
        DefRevealed::TypeDef { sym, def } => DefAtomized::TypeDef { sym, def },
    }
}

fn atomize_expr<'p>(expr: Typed<'p, RExpr<'p>>) -> Typed<'p, AExpr<'p>> {
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
                atomize_atom(*lhs, &mut priors),
                atomize_atom(*rhs, &mut priors),
            ],
        },
        RExpr::UnaryOp { op, expr: arg } => AExpr::UnaryOp {
            op,
            expr: atomize_atom(*arg, &mut priors),
        },
        RExpr::Let { sym, bnd, bdy } => AExpr::Let {
            sym,
            bnd: Box::new(atomize_expr(*bnd)),
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::If { cnd, thn, els } => AExpr::If {
            cnd: Box::new(atomize_expr(*cnd)),
            thn: Box::new(atomize_expr(*thn)),
            els: Box::new(atomize_expr(*els)),
        },
        RExpr::Apply { fun, args } => {
            let Type::Fn { params, .. } = fun.meta.clone() else {
                unreachable!()
            };

            AExpr::Apply {
                fun: atomize_atom(*fun, &mut priors),
                args: args
                    .into_iter()
                    .map(|arg| atomize_atom(arg, &mut priors))
                    .zip(params)
                    .collect(),
            }
        }
        RExpr::FunRef { sym } => AExpr::FunRef { sym },
        RExpr::Loop { bdy } => AExpr::Loop {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Break { bdy } => AExpr::Break {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Seq { stmt, cnt } => AExpr::Seq {
            stmt: Box::new(atomize_expr(*stmt)),
            cnt: Box::new(atomize_expr(*cnt)),
        },
        RExpr::Assign { sym, bnd } => AExpr::Assign {
            sym,
            bnd: Box::new(atomize_expr(*bnd)),
        },
        RExpr::Continue => AExpr::Continue,
        RExpr::Return { bdy } => AExpr::Return {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Struct { sym, fields } => AExpr::Struct {
            sym,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| {
                    let field = atomize_atom(expr, &mut priors);
                    (sym, field)
                })
                .collect(),
        },
        RExpr::AccessField { strct, field } => AExpr::AccessField {
            strct: atomize_atom(*strct, &mut priors),
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
) -> Atom<'p> {
    match expr.inner {
        RExpr::Lit { val } => Atom::Val { val },
        RExpr::Var { sym} => Atom::Var { sym },
        _ => {
            let tmp = gen_sym("tmp");
            priors.push((tmp, atomize_expr(expr)));
            Atom::Var { sym: tmp }
        }
    }
}