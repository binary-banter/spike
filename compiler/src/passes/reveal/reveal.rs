use crate::passes::parse::{Meta, Typed};
use crate::passes::reveal::{DefRevealed, PrgRevealed, RExpr};
use crate::passes::validate::{DefValidated, ExprValidated, PrgValidated};
use crate::utils::push_map::PushMap;
use crate::utils::unique_sym::UniqueSym;
use crate::{display, time};

impl<'p> PrgValidated<'p> {
    #[must_use]
    pub fn reveal(self) -> PrgRevealed<'p> {
        let mut scope = PushMap::from_iter(self.defs.keys().map(|s| (*s, ())));

        let program = PrgRevealed {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, reveal_def(def, &mut scope)))
                .collect(),
            entry: self.entry,
        };

        display!(&program, Reveal);
        time!("reveal");

        program
    }
}

fn reveal_def<'p>(
    def: DefValidated<'p>,
    scope: &mut PushMap<UniqueSym<'p>, ()>,
) -> DefRevealed<'p> {
    match def {
        DefValidated::Fn {
            sym,
            params,
            typ,
            bdy,
        } => DefRevealed::Fn {
            sym,
            params,
            typ,
            bdy: reveal_expr(bdy, scope),
        },
        DefValidated::TypeDef { sym, def } => DefRevealed::TypeDef { sym, def },
    }
}

fn reveal_expr<'p>(
    expr: Typed<'p, ExprValidated<'p>>,
    scope: &mut PushMap<UniqueSym<'p>, ()>,
) -> Typed<'p, RExpr<'p>> {
    let inner = match expr.inner {
        ExprValidated::Lit { val } => RExpr::Lit { val },
        ExprValidated::Var { sym } => {
            if scope.contains(&sym) {
                RExpr::FunRef { sym }
            } else {
                RExpr::Var { sym }
            }
        }
        ExprValidated::UnaryOp { op, expr } => RExpr::UnaryOp {
            op,
            expr: Box::new(reveal_expr(*expr, scope)),
        },
        ExprValidated::BinaryOp {
            op,
            exprs: [lhs, rhs],
        } => RExpr::BinaryOp {
            op,
            exprs: [
                Box::new(reveal_expr(*lhs, scope)),
                Box::new(reveal_expr(*rhs, scope)),
            ],
        },
        ExprValidated::Let {
            sym,
            mutable,
            bnd,
            bdy,
            ..
        } => {
            let bnd = Box::new(reveal_expr(*bnd, scope));
            scope.remove(sym, |scope| RExpr::Let {
                sym,
                mutable,
                bnd,
                bdy: Box::new(reveal_expr(*bdy, scope)),
            })
        }
        ExprValidated::If { cnd, thn, els } => RExpr::If {
            cnd: Box::new(reveal_expr(*cnd, scope)),
            thn: Box::new(reveal_expr(*thn, scope)),
            els: Box::new(reveal_expr(*els, scope)),
        },
        ExprValidated::Apply { fun, args } => RExpr::Apply {
            fun: Box::new(reveal_expr(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
        },
        ExprValidated::Loop { bdy } => RExpr::Loop {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        ExprValidated::Break { bdy } => RExpr::Break {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        ExprValidated::Seq { stmt, cnt } => RExpr::Seq {
            stmt: Box::new(reveal_expr(*stmt, scope)),
            cnt: Box::new(reveal_expr(*cnt, scope)),
        },
        ExprValidated::Assign { sym, bnd } => RExpr::Assign {
            sym,
            bnd: Box::new(reveal_expr(*bnd, scope)),
        },
        ExprValidated::Continue => RExpr::Continue,
        ExprValidated::Return { bdy } => RExpr::Return {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        ExprValidated::Struct { sym, fields } => RExpr::Struct {
            sym,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| (sym, reveal_expr(expr, scope)))
                .collect(),
        },
        ExprValidated::AccessField { strct, field } => RExpr::AccessField {
            strct: Box::new(reveal_expr(*strct, scope)),
            field,
        },
        ExprValidated::Variant { .. } => todo!(),
        ExprValidated::Switch { .. } => todo!(),
        ExprValidated::Asm { instrs } => RExpr::Asm { instrs },
    };

    Meta {
        meta: expr.meta,
        inner,
    }
}
