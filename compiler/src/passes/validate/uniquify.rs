use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr, Param, PrgParsed, Spanned, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::{NoMain, UndeclaredVar};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

#[derive(Debug, PartialEq)]
pub struct PrgUniquified<'p> {
    /// The global program definitions.
    pub defs: Vec<
        Def<
            Spanned<UniqueSym<'p>>,
            Spanned<&'p str>,
            Spanned<Expr<'p, Spanned<UniqueSym<'p>>, Spanned<&'p str>>>,
        >,
    >,
    /// The symbol representing the entry point of the program.
    pub entry: UniqueSym<'p>,
}

impl<'p> PrgParsed<'p> {
    pub fn uniquify(self) -> Result<PrgUniquified<'p>, TypeError> {
        todo!()
        // let mut scope =
        //     PushMap::from_iter(self.defs.iter().map(|def| (*def.sym(), gen_sym(def.sym()))));
        //
        // let entry = *scope.get(&"main").ok_or(NoMain)?;
        //
        // Ok(PrgUniquified {
        //     defs: self
        //         .defs
        //         .into_iter()
        //         .map(|def| uniquify_def(def, &mut scope))
        //         .collect::<Result<_, _>>()?,
        //     entry,
        // })
    }
}

fn uniquify_def<'p>(
    def: Def<
        Spanned<&'p str>,
        Spanned<&'p str>,
        Spanned<Expr<'p, Spanned<&'p str>, Spanned<&'p str>>>,
    >,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<
    Def<
        Spanned<UniqueSym<'p>>,
        Spanned<&'p str>,
        Spanned<Expr<'p, Spanned<UniqueSym<'p>>, Spanned<&'p str>>>,
    >,
    TypeError,
> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => {
            scope.push_iter(
                params
                    .iter()
                    .map(|param| (param.sym.inner, gen_spanned_sym(param.sym).inner)),
                |scope| {
                    todo!()
                    //         let params = params
                    //             .iter()
                    //             .map(|param| Param {
                    //                 sym: scope[&param.sym],
                    //                 mutable: param.mutable,
                    //                 typ: param.typ.clone().fmap(|v| scope[v]),
                    //             })
                    //             .collect();
                    //         let bdy = uniquify_expression(bdy, scope)?;
                    //
                    //         Ok(Def::Fn {
                    //             sym: scope[&sym],
                    //             params,
                    //             typ: typ.fmap(|v| scope[v]),
                    //             bdy,
                    //         })
                },
            )
        }
        Def::TypeDef { sym, def } => {
            let def = match def {
                TypeDef::Struct { fields } => TypeDef::Struct {
                    fields: fields
                        .into_iter()
                        .map(|(sym, typ)| Ok((sym, uniquify_type(typ, scope)?)))
                        .collect::<Result<_, _>>()?,
                },
                TypeDef::Enum { .. } => todo!(),
            };

            Ok(Def::TypeDef {
                sym: try_get(sym, scope)?,
                def,
            })
        }
    }
}

fn uniquify_type<'p>(
    typ: Type<Spanned<&'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Type<Spanned<UniqueSym<'p>>>, TypeError> {
    let typ = match typ {
        Type::Int => Type::Int,
        Type::Bool => Type::Bool,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Fn { params, typ } => Type::Fn {
            params: params
                .into_iter()
                .map(|param| uniquify_type(param, scope))
                .collect::<Result<_, _>>()?,
            typ: Box::new(uniquify_type(*typ, scope)?),
        },
        Type::Var { sym } => Type::Var { sym: try_get(sym, scope)? },
    };

    Ok(typ)
}

fn uniquify_expression<'p>(
    expr: Spanned<Expr<'p, Spanned<&'p str>, Spanned<&'p str>>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Spanned<Expr<'p, Spanned<UniqueSym<'p>>, Spanned<&'p str>>>, TypeError> {
    let inner: Expr<'p, Spanned<UniqueSym<'p>>, Spanned<&'p str>> = match expr.inner {
        Expr::Let {
            sym,
            bnd,
            bdy,
            mutable,
        } => {
            let unique_bnd = uniquify_expression(*bnd, scope)?;
            let unique_sym = gen_spanned_sym(sym);
            let unique_bdy = scope.push(sym.inner, unique_sym.inner, |scope| {
                uniquify_expression(*bdy, scope)
            })?;

            Expr::Let {
                sym: unique_sym,
                mutable,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::Var { sym } => Expr::Var {
            sym: try_get(sym, scope)?,
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: try_get(sym, scope)?,
            bnd: Box::new(uniquify_expression(*bnd, scope)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: try_get(sym, scope)?,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| uniquify_expression(expr, scope).map(|expr| (sym, expr)))
                .collect::<Result<_, _>>()?,
        },

        Expr::Lit { val } => Expr::Lit { val },
        Expr::Prim { op, args } => Expr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect::<Result<_, _>>()?,
        },
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(uniquify_expression(*cnd, scope)?),
            thn: Box::new(uniquify_expression(*thn, scope)?),
            els: Box::new(uniquify_expression(*els, scope)?),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(uniquify_expression(*fun, scope)?),
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect::<Result<_, _>>()?,
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(uniquify_expression(*bdy, scope)?),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: Box::new(uniquify_expression(*bdy, scope)?),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(uniquify_expression(*stmt, scope)?),
            cnt: Box::new(uniquify_expression(*cnt, scope)?),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(uniquify_expression(*bdy, scope)?),
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(uniquify_expression(*strct, scope)?),
            field,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    };

    Ok(Spanned {
        inner,
        span: expr.span,
    })
}

fn try_get<'p>(
    sym: Spanned<&'p str>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Spanned<UniqueSym<'p>>, TypeError> {
    scope
        .get(&sym.inner)
        .ok_or(UndeclaredVar {
            sym: sym.inner.to_string(),
            span: sym.span,
        })
        .map(|&inner| Spanned {
            span: sym.span,
            inner,
        })
}

fn gen_spanned_sym(sym: Spanned<&str>) -> Spanned<UniqueSym> {
    Spanned {
        inner: gen_sym(sym.inner),
        span: sym.span,
    }
}
