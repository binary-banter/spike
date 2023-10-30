use crate::passes::parse::{Def, Expr};
use crate::passes::reveal_functions::{PrgRevealed, RExpr};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

impl<'p> PrgUniquified<'p> {
    #[must_use]
    pub fn reveal(self) -> PrgRevealed<'p> {
        let mut scope = PushMap::from_iter(self.defs.keys().map(|s| (*s, ())));

        PrgRevealed {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| {
                    let def = match def {
                        Def::Fn {
                            sym,
                            params,
                            typ,
                            bdy,
                        } => Def::Fn {
                            sym,
                            params,
                            typ,
                            bdy: reveal_expr(bdy, &mut scope),
                        },
                    };

                    (sym, def)
                })
                .collect(),
            entry: self.entry,
        }
    }
}

fn reveal_expr<'p>(expr: Expr<UniqueSym<'p>>, scope: &mut PushMap<UniqueSym<'p>, ()>) -> RExpr<'p> {
    match expr {
        Expr::Lit { val } => RExpr::Lit { val },
        Expr::Var { sym } => {
            if scope.contains(&sym) {
                RExpr::FunRef { sym }
            } else {
                RExpr::Var { sym }
            }
        }
        Expr::Prim { op, args } => RExpr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
        },
        Expr::Let { sym, bnd, bdy, .. } => {
            let bnd = Box::new(reveal_expr(*bnd, scope));
            scope.remove(sym, |scope| RExpr::Let {
                sym,
                bnd,
                bdy: Box::new(reveal_expr(*bdy, scope)),
            })
        }
        Expr::If { cnd, thn, els } => RExpr::If {
            cnd: Box::new(reveal_expr(*cnd, scope)),
            thn: Box::new(reveal_expr(*thn, scope)),
            els: Box::new(reveal_expr(*els, scope)),
        },
        Expr::Apply { fun, args } => RExpr::Apply {
            fun: Box::new(reveal_expr(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
        },
        Expr::Loop { bdy } => RExpr::Loop {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        Expr::Break { bdy } => RExpr::Break {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        Expr::Seq { stmt, cnt } => RExpr::Seq {
            stmt: Box::new(reveal_expr(*stmt, scope)),
            cnt: Box::new(reveal_expr(*cnt, scope)),
        },
        Expr::Assign { sym, bnd } => RExpr::Assign {
            sym,
            bnd: Box::new(reveal_expr(*bnd, scope)),
        },
    }
}
