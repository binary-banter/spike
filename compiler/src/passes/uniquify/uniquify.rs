use crate::passes::parse::{Def, Expr, Param};
use crate::passes::type_check::PrgTypeChecked;
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

impl<'p> PrgTypeChecked<'p> {
    #[must_use]
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
    def: Def<'p, &'p str, Expr<'p, &'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Def<'p, UniqueSym<'p>, Expr<'p, UniqueSym<'p>>> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => scope.push_iter(
            params.iter().map(|param| (param.sym, gen_sym(param.sym))),
            |scope| {
                let params = params
                    .iter()
                    .map(|param| Param {
                        sym: scope[&param.sym],
                        mutable: param.mutable,
                        typ: param.typ.clone().fmap(|v| scope[v]),
                    })
                    .collect();
                let bdy = uniquify_expression(bdy, scope);
                Def::Fn {
                    sym: scope[&sym],
                    params,
                    typ: typ.fmap(|v| scope[v]),
                    bdy,
                }
            },
        ),
        Def::Struct { sym, fields } => Def::Struct {
            sym: scope[&sym],
            fields: fields
                .into_iter()
                .map(|(sym, typ)| (sym, typ.fmap(|sym| scope[sym])))
                .collect(),
        },
        Def::Enum { .. } => todo!(),
    }
}

fn uniquify_expression<'p>(
    expr: Expr<'p, &'p str>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Expr<'p, UniqueSym<'p>> {
    match expr {
        Expr::Let {
            sym,
            mutable,
            bnd,
            bdy,
        } => {
            let unique_bnd = uniquify_expression(*bnd, scope);
            let unique_sym = gen_sym(sym);
            let unique_bdy = scope.push(sym, unique_sym, |scope| uniquify_expression(*bdy, scope));

            Expr::Let {
                sym: unique_sym,
                mutable,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::Var { sym } => Expr::Var { sym: scope[&sym] },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: scope[sym],
            bnd: Box::new(uniquify_expression(*bnd, scope)),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: scope[sym],
            fields: fields
                .into_iter()
                .map(|(sym, expr)| (sym, uniquify_expression(expr, scope)))
                .collect(),
        },

        Expr::Lit { val } => Expr::Lit { val },
        Expr::Prim { op, args } => Expr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
        },
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
            bdy: Box::new(uniquify_expression(*bdy, scope)),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(uniquify_expression(*stmt, scope)),
            cnt: Box::new(uniquify_expression(*cnt, scope)),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(uniquify_expression(*bdy, scope)),
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(uniquify_expression(*strct, scope)),
            field,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    }
}
