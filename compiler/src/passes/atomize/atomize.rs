use crate::passes::atomize::{AExpr, Atom, DefAtomized, PrgAtomized};
use crate::passes::parse::types::Type;
use crate::passes::parse::Meta;
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

fn atomize_expr<'p>(expr: Meta<Type<UniqueSym<'p>>, RExpr<'p>>) -> Meta<Type<UniqueSym<'p>>, AExpr<'p>> {
    let inner = match expr.inner {
        RExpr::Lit { val } => AExpr::Atom {
            atm: Atom::Val { val },
        },
        RExpr::Var { sym} => AExpr::Atom {
            atm: Atom::Var { sym },
        },
        RExpr::BinaryOp { op, exprs: [lhs, rhs] } => {
            let (lhs_arg, lhs_extra) = atomize_atom(*lhs);
            let (rhs_arg, rhs_extra) = atomize_atom(*rhs);

            [lhs_extra, rhs_extra]
                .into_iter()
                .flatten()
                .rfold(AExpr::BinaryOp { op, exprs: [lhs_arg, rhs_arg] }, |bdy, (sym, bnd)| {
                    AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(Meta {
                            inner: bdy,
                            meta: expr.meta.clone()
                        }),
                    }
                })
        },
        RExpr::UnaryOp { op, expr: arg } => {
            let (arg, extra) = atomize_atom(*arg);

            extra
                .into_iter()
                .rfold(AExpr::UnaryOp { op, expr: arg }, |bdy, (sym, bnd)| {
                    AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(Meta {
                            inner: bdy,
                            meta: expr.meta.clone()
                        }),
                    }
                })
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

            let (args, extras): (Vec<_>, Vec<_>) = args
                .into_iter()
                .map(atomize_atom)
                .zip(params)
                .map(|((arg, extra), arg_typ)| ((arg, arg_typ), extra))
                .unzip();

            let (fun, fun_expr) = atomize_atom(*fun);

            fun_expr
                .into_iter()
                .chain(extras.into_iter().flatten())
                .rfold(AExpr::Apply { fun, args}, |bdy, (sym, bnd)| {
                    AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(Meta {
                            inner: bdy,
                            meta: expr.meta.clone()
                        }),
                    }
                })
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
        RExpr::Struct { sym, fields } => {
            let (fields, extras): (Vec<_>, Vec<_>) = fields
                .into_iter()
                .map(|(sym, expr)| {
                    let (field, extra) = atomize_atom(expr);
                    ((sym, field), extra)
                })
                .unzip();

            extras.into_iter().flatten().rfold(
                AExpr::Struct { sym, fields },
                |bdy , (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(Meta {
                        inner: bdy,
                        meta: expr.meta.clone()
                    }),
                },
            )
        }
        RExpr::AccessField { strct, field } => {
            let (strct, extra) = atomize_atom(*strct);

            extra.into_iter().rfold(
                AExpr::AccessField { strct, field },
                |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(Meta {
                        inner: bdy,
                        meta: expr.meta.clone()
                    }),
                },
            )
        }
    };

    Meta {
        inner,
        meta: expr.meta
    }
}

fn atomize_atom<'p>(expr: Meta<Type<UniqueSym<'p>>, RExpr<'p>>) -> (Atom<'p>, Option<(UniqueSym<'p>, Meta<Type<UniqueSym<'p>>, AExpr<'p>>)>) {
    if let RExpr::Lit { val } = expr.inner {
        (Atom::Val { val }, None)
    } else {
        let tmp = gen_sym("tmp");
        (Atom::Var { sym: tmp }, Some((tmp, atomize_expr(expr))))
    }
}
