use crate::passes::parse::Def;
use crate::passes::reveal_functions::{PrgRevealed, RExpr};
use crate::passes::type_check::TExpr;
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

impl<'p> PrgUniquified<'p> {
    #[must_use]
    pub fn reveal(self) -> PrgRevealed<'p> {

        todo!()
        // let mut scope = PushMap::from_iter(self.defs.keys().map(|s| (*s, ())));
        // PrgRevealed {
        //     defs: self
        //         .defs
        //         .into_iter()
        //         .map(|(sym, def)| {
        //             let def = match def {
        //                 Def::Fn {
        //                     sym,
        //                     params,
        //                     typ,
        //                     bdy,
        //                 } => Def::Fn {
        //                     sym,
        //                     params,
        //                     typ,
        //                     bdy: reveal_expr(bdy, &mut scope),
        //                 },
        //                 Def::Struct { sym, fields } => Def::Struct { sym, fields },
        //                 Def::Enum { sym, variants } => Def::Enum { sym, variants },
        //             };
        //
        //             (sym, def)
        //         })
        //         .collect(),
        //     entry: self.entry,
        // }
    }
}

fn reveal_expr<'p>(
    expr: TExpr<'p, UniqueSym<'p>>,
    scope: &mut PushMap<UniqueSym<'p>, ()>,
) -> RExpr<'p> {
    match expr {
        TExpr::Lit { val , ..} => RExpr::Lit { val },
        TExpr::Var { sym, .. } => {
            if scope.contains(&sym) {
                RExpr::FunRef { sym }
            } else {
                RExpr::Var { sym }
            }
        }
        TExpr::Prim { op, args, .. } => RExpr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
        },
        TExpr::Let { sym, bnd, bdy, .. } => {
            let bnd = Box::new(reveal_expr(*bnd, scope));
            scope.remove(sym, |scope| RExpr::Let {
                sym,
                bnd,
                bdy: Box::new(reveal_expr(*bdy, scope)),
            })
        }
        TExpr::If { cnd, thn, els, .. } => RExpr::If {
            cnd: Box::new(reveal_expr(*cnd, scope)),
            thn: Box::new(reveal_expr(*thn, scope)),
            els: Box::new(reveal_expr(*els, scope)),
        },
        TExpr::Apply { fun, args, .. } => RExpr::Apply {
            fun: Box::new(reveal_expr(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
        },
        TExpr::Loop { bdy, .. } => RExpr::Loop {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        TExpr::Break { bdy, .. } => RExpr::Break {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        TExpr::Seq { stmt, cnt, .. } => RExpr::Seq {
            stmt: Box::new(reveal_expr(*stmt, scope)),
            cnt: Box::new(reveal_expr(*cnt, scope)),
        },
        TExpr::Assign { sym, bnd, .. } => RExpr::Assign {
            sym,
            bnd: Box::new(reveal_expr(*bnd, scope)),
        },
        TExpr::Continue{..} => RExpr::Continue,
        TExpr::Return { bdy, .. } => RExpr::Return {
            bdy: Box::new(reveal_expr(*bdy, scope)),
        },
        TExpr::Struct { sym, fields, .. } => RExpr::Struct {
            sym,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| (sym, reveal_expr(expr, scope)))
                .collect(),
        },
        TExpr::AccessField { strct, field, .. } => RExpr::AccessField {
            strct: Box::new(reveal_expr(*strct, scope)),
            field,
        },
        TExpr::Variant { .. } => todo!(),
        TExpr::Switch { .. } => todo!(),
    }
}
