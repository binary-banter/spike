use crate::passes::parse::Def;
use crate::passes::reveal_functions::{PrgRevealed, RExpr};
use crate::passes::validate::{PrgValidated, TExpr};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

impl<'p> PrgValidated<'p> {
    #[must_use]
    pub fn reveal(self) -> PrgRevealed<'p> {
        todo!()

        // let mut scope = PushMap::from_iter(self.defs.keys().map(|s| (*s, ())));
        // PrgRevealed {
        //     defs: self
        //         .defs
        //         .into_iter()
        //         .map(|(sym, def)| (sym, reveal_def(def, &mut scope)))
        //         .collect(),
        //     entry: self.entry,
        // }
    }
}

fn reveal_def<'p>(
    def: Def<'p, UniqueSym<'p>, TExpr<'p, UniqueSym<'p>>>,
    scope: &mut PushMap<UniqueSym<'p>, ()>,
) -> Def<'p, UniqueSym<'p>, RExpr<'p>> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => Def::Fn {
            sym,
            params,
            typ,
            bdy: reveal_expr(bdy, scope),
        },
        Def::TypeDef { sym, def } => Def::TypeDef { sym, def },
    }
}

fn reveal_expr<'p>(
    expr: TExpr<'p, UniqueSym<'p>>,
    scope: &mut PushMap<UniqueSym<'p>, ()>,
) -> RExpr<'p> {
    match expr {
        TExpr::Lit { val, typ } => RExpr::Lit { val, typ },
        TExpr::Var { sym, typ } => {
            if scope.contains(&sym) {
                RExpr::FunRef { sym, typ }
            } else {
                RExpr::Var { sym, typ }
            }
        }
        TExpr::Prim { op, args, typ } => RExpr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
            typ,
        },
        TExpr::Let {
            sym, bnd, bdy, typ, ..
        } => {
            let bnd = Box::new(reveal_expr(*bnd, scope));
            scope.remove(sym, |scope| RExpr::Let {
                sym,
                bnd,
                bdy: Box::new(reveal_expr(*bdy, scope)),
                typ,
            })
        }
        TExpr::If { cnd, thn, els, typ } => RExpr::If {
            cnd: Box::new(reveal_expr(*cnd, scope)),
            thn: Box::new(reveal_expr(*thn, scope)),
            els: Box::new(reveal_expr(*els, scope)),
            typ,
        },
        TExpr::Apply { fun, args, typ } => RExpr::Apply {
            fun: Box::new(reveal_expr(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| reveal_expr(arg, scope))
                .collect(),
            typ,
        },
        TExpr::Loop { bdy, typ } => RExpr::Loop {
            bdy: Box::new(reveal_expr(*bdy, scope)),
            typ,
        },
        TExpr::Break { bdy, typ } => RExpr::Break {
            bdy: Box::new(reveal_expr(*bdy, scope)),
            typ,
        },
        TExpr::Seq { stmt, cnt, typ } => RExpr::Seq {
            stmt: Box::new(reveal_expr(*stmt, scope)),
            cnt: Box::new(reveal_expr(*cnt, scope)),
            typ,
        },
        TExpr::Assign { sym, bnd, typ } => RExpr::Assign {
            sym,
            bnd: Box::new(reveal_expr(*bnd, scope)),
            typ,
        },
        TExpr::Continue { typ } => RExpr::Continue { typ },
        TExpr::Return { bdy, typ } => RExpr::Return {
            bdy: Box::new(reveal_expr(*bdy, scope)),
            typ,
        },
        TExpr::Struct { sym, fields, typ } => RExpr::Struct {
            sym,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| (sym, reveal_expr(expr, scope)))
                .collect(),
            typ,
        },
        TExpr::AccessField { strct, field, typ } => RExpr::AccessField {
            strct: Box::new(reveal_expr(*strct, scope)),
            field,
            typ,
        },
        TExpr::Variant { .. } => todo!(),
        TExpr::Switch { .. } => todo!(),
    }
}
