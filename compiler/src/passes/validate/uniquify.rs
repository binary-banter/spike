use std::collections::HashMap;
use once_cell::sync::{Lazy, OnceCell};
use crate::passes::parse::types::Type;
use crate::passes::parse::{
    Def, DefParsed, Expr, ExprParsed, Meta, Param, PrgParsed, Span, TypeDef,
};
use crate::passes::select::io::Std;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::{NoMain, UndeclaredVar};
use crate::passes::validate::{DefUniquified, ExprUniquified};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;

pub struct PrgUniquified<'p> {
    /// The global program definitions.
    pub defs: Vec<DefUniquified<'p>>,
    /// The symbol representing the entry point of the program.
    pub entry: UniqueSym<'p>,

    pub std: Std<'p>,
}

pub static BUILT_INS: Lazy<HashMap<&'static str, Type<Meta<Span, UniqueSym<'static>>>>> = Lazy::new(|| {
    HashMap::from([
        ("exit", Type::Fn { params: vec![Type::I64], typ: Box::new(Type::Never) }),
        ("print", Type::Fn { params: vec![Type::I64], typ: Box::new(Type::I64) }),
        ("read", Type::Fn { params: vec![], typ: Box::new(Type::I64) })
    ])
});

impl<'p> PrgParsed<'p> {
    pub fn uniquify(self) -> Result<PrgUniquified<'p>, TypeError> {
        let std: Std<'p> = BUILT_INS.iter().map(|(sym, _)| (*sym, gen_sym(sym))).collect();

        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|def| (def.sym().inner, gen_sym(def.sym().inner)))
                .chain(std.iter().map(|(&k, &v)| (k, v)))
        );

        let entry = *scope.get(&"main").ok_or(NoMain)?;

        Ok(PrgUniquified {
            defs: self
                .defs
                .into_iter()
                .map(|def| uniquify_def(def, &mut scope))
                .collect::<Result<_, _>>()?,
            entry,
            std
        })
    }
}


fn uniquify_def<'p>(
    def: DefParsed<'p>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<DefUniquified<'p>, TypeError> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => scope.push_iter::<Result<_, _>>(
            params
                .iter()
                .map(|param| (param.sym.inner, gen_spanned_sym(param.sym.clone()).inner)),
            |scope| {
                let params = params
                    .iter()
                    .map(|param| uniquify_param(param, scope))
                    .collect::<Result<_, _>>()?;
                let bdy = uniquify_expr(bdy, scope)?;

                //TODO check if function names and param names are unique

                Ok(Def::Fn {
                    sym: try_get(sym, scope)?,
                    params,
                    typ: uniquify_type(typ, scope)?,
                    bdy,
                })
            },
        ),
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

fn uniquify_param<'p>(
    param: &Param<Meta<Span, &'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Param<Meta<Span, UniqueSym<'p>>>, TypeError> {
    Ok(Param {
        sym: try_get(param.sym.clone(), scope)?,
        mutable: param.mutable,
        typ: uniquify_type(param.typ.clone(), scope)?,
    })
}

fn uniquify_type<'p>(
    typ: Type<Meta<Span, &'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Type<Meta<Span, UniqueSym<'p>>>, TypeError> {
    let typ = match typ {
        Type::I64 => Type::I64,
        Type::U64 => Type::U64,
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
        Type::Var { sym } => Type::Var {
            sym: try_get(sym, scope)?,
        },
    };

    Ok(typ)
}

fn uniquify_expr<'p>(
    expr: Meta<Span, ExprParsed<'p>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Meta<Span, ExprUniquified<'p>>, TypeError> {
    let inner = match expr.inner {
        Expr::Let {
            sym,
            typ,
            bnd,
            bdy,
            mutable,
        } => {
            let unique_bnd = uniquify_expr(*bnd, scope)?;
            let unique_sym = gen_spanned_sym(sym.clone());
            let unique_bdy = scope.push(sym.inner, unique_sym.inner, |scope| {
                uniquify_expr(*bdy, scope)
            })?;

            Expr::Let {
                sym: unique_sym,
                mutable,
                typ: typ.map(|typ| uniquify_type(typ, scope)).transpose()?,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::Var { sym } => Expr::Var {
            sym: try_get(sym, scope)?,
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: try_get(sym, scope)?,
            bnd: Box::new(uniquify_expr(*bnd, scope)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: try_get(sym, scope)?,
            fields: fields
                .into_iter()
                .map(|(sym, expr)| uniquify_expr(expr, scope).map(|expr| (sym, expr)))
                .collect::<Result<_, _>>()?,
        },

        Expr::Lit { val } => Expr::Lit { val },
        Expr::UnaryOp { op, expr } => Expr::UnaryOp {
            op,
            expr: Box::new(uniquify_expr(*expr, scope)?),
        },
        Expr::BinaryOp {
            op,
            exprs: [e1, e2],
        } => Expr::BinaryOp {
            op,
            exprs: [uniquify_expr(*e1, scope)?, uniquify_expr(*e2, scope)?].map(Box::new),
        },
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(uniquify_expr(*cnd, scope)?),
            thn: Box::new(uniquify_expr(*thn, scope)?),
            els: Box::new(uniquify_expr(*els, scope)?),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(uniquify_expr(*fun, scope)?),
            args: args
                .into_iter()
                .map(|arg| uniquify_expr(arg, scope))
                .collect::<Result<_, _>>()?,
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(uniquify_expr(*stmt, scope)?),
            cnt: Box::new(uniquify_expr(*cnt, scope)?),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(uniquify_expr(*bdy, scope)?),
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(uniquify_expr(*strct, scope)?),
            field,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    };

    Ok(Meta {
        inner,
        meta: expr.meta,
    })
}

fn try_get<'p>(
    sym: Meta<Span, &'p str>,
    scope: &PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Meta<Span, UniqueSym<'p>>, TypeError> {
    scope
        .get(&sym.inner)
        .ok_or(UndeclaredVar {
            sym: sym.inner.to_string(),
            span: sym.meta,
        })
        .map(|&inner| Meta {
            meta: sym.meta,
            inner,
        })
}

fn gen_spanned_sym(sym: Meta<Span, &str>) -> Meta<Span, UniqueSym> {
    Meta {
        inner: gen_sym(sym.inner),
        meta: sym.meta,
    }
}
