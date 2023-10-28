use crate::passes::parse::{Def, Expr};
use crate::passes::type_check::PrgTypeChecked;
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

impl<'p> PrgTypeChecked<'p> {
    pub fn uniquify(self) -> PrgUniquified<'p> {
        let mut scope = PushMap::from_iter(self.defs.iter().map(|(&sym, _)| (sym, gen_sym(sym))));

        PrgUniquified {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| (scope[&sym], uniquify_def(def, &mut scope)))
                .collect(),
            entry: scope[&self.entry],
        }
    }
}

fn uniquify_def<'p>(
    def: Def<&'p str, Expr<&'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Def<UniqueSym<'p>, Expr<UniqueSym<'p>>> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => scope.push_iter(
            params.iter().map(|(sym, _)| (*sym, gen_sym(sym))),
            |scope| {
                let params = params
                    .iter()
                    .cloned()
                    .map(|(p, t)| (scope[&p], t))
                    .collect();
                let bdy = uniquify_expression(bdy, scope);
                Def::Fn {
                    sym: scope[&sym],
                    params,
                    typ,
                    bdy,
                }
            },
        ),
    }
}

fn uniquify_expression<'p>(
    expr: Expr<&'p str>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Expr<UniqueSym<'p>> {
    match expr {
        Expr::Lit { val } => Expr::Lit { val },
        Expr::Var { sym } => Expr::Var { sym: scope[&sym] },
        Expr::Prim { op, args } => Expr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
        },
        Expr::Let { sym, bnd, bdy } => {
            let unique_bnd = uniquify_expression(*bnd, scope);
            let unique_sym = gen_sym(sym);
            let unique_bdy = scope.push(sym, unique_sym, |scope| uniquify_expression(*bdy, scope));

            Expr::Let {
                sym: unique_sym,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(uniquify_expression(*cnd, scope)),
            thn: Box::new(uniquify_expression(*thn, scope)),
            els: Box::new(uniquify_expression(*els, scope)),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(uniquify_expression(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(uniquify_expression(*bdy, scope)),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: bdy.map(|bdy| Box::new(uniquify_expression(*bdy, scope))),
        },
    }
}