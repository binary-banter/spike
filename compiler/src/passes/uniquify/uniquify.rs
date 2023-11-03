use crate::passes::parse::{Def, Param, TypeDef};
use crate::passes::type_check::{PrgTypeChecked, TExpr};
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
    def: Def<'p, &'p str, TExpr<'p, &'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Def<'p, UniqueSym<'p>, TExpr<'p, UniqueSym<'p>>> {
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
        Def::TypeDef { sym, def } => {
            let def = match def {
                TypeDef::Struct { fields } => TypeDef::Struct {
                    fields: fields
                        .into_iter()
                        .map(|(sym, typ)| (sym, typ.fmap(|sym| scope[sym])))
                        .collect(),
                },
                TypeDef::Enum { .. } => todo!(),
            };
            Def::TypeDef {
                sym: scope[&sym],
                def,
            }
        }
    }
}

fn uniquify_expression<'p>(
    expr: TExpr<'p, &'p str>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> TExpr<'p, UniqueSym<'p>> {
    match expr {
        TExpr::Let { sym, bnd, bdy, typ } => {
            let unique_bnd = uniquify_expression(*bnd, scope);
            let unique_sym = gen_sym(sym);
            let unique_bdy = scope.push(sym, unique_sym, |scope| uniquify_expression(*bdy, scope));

            TExpr::Let {
                sym: unique_sym,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
                typ: typ.fmap(|s| scope[s]),
            }
        }
        TExpr::Var { sym, typ } => TExpr::Var {
            sym: scope[&sym],
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Assign { sym, bnd, typ } => TExpr::Assign {
            sym: scope[sym],
            bnd: Box::new(uniquify_expression(*bnd, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Struct { sym, fields, typ } => TExpr::Struct {
            sym: scope[sym],
            fields: fields
                .into_iter()
                .map(|(sym, TExpr)| (sym, uniquify_expression(TExpr, scope)))
                .collect(),
            typ: typ.fmap(|s| scope[s]),
        },

        TExpr::Lit { val, typ } => TExpr::Lit {
            val,
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Prim { op, args, typ } => TExpr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::If { cnd, thn, els, typ } => TExpr::If {
            cnd: Box::new(uniquify_expression(*cnd, scope)),
            thn: Box::new(uniquify_expression(*thn, scope)),
            els: Box::new(uniquify_expression(*els, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Apply { fun, args, typ } => TExpr::Apply {
            fun: Box::new(uniquify_expression(*fun, scope)),
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Loop { bdy, typ } => TExpr::Loop {
            bdy: Box::new(uniquify_expression(*bdy, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Break { bdy, typ } => TExpr::Break {
            bdy: Box::new(uniquify_expression(*bdy, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Seq { stmt, cnt, typ } => TExpr::Seq {
            stmt: Box::new(uniquify_expression(*stmt, scope)),
            cnt: Box::new(uniquify_expression(*cnt, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Continue { typ } => TExpr::Continue {
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Return { bdy, typ } => TExpr::Return {
            bdy: Box::new(uniquify_expression(*bdy, scope)),
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::AccessField { strct, field, typ } => TExpr::AccessField {
            strct: Box::new(uniquify_expression(*strct, scope)),
            field,
            typ: typ.fmap(|s| scope[s]),
        },
        TExpr::Variant { .. } => todo!(),
        TExpr::Switch { .. } => todo!(),
    }
}
