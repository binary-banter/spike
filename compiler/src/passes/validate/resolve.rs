use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Lit, Meta, Param, Span, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{
    CMeta, DefConstrained, DefValidated, ExprConstrained, ExprValidated, PrgConstrained,
    PrgValidated, TLit,
};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use functor_derive::Functor;

impl<'p> PrgConstrained<'p> {
    pub fn resolve(mut self) -> Result<PrgValidated<'p>, TypeError> {
        Ok(PrgValidated {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| resolve_def(def, &mut self.uf).map(|def| (sym, def)))
                .collect::<Result<_, _>>()?,
            entry: self.entry,
            std: self.std,
        })
    }
}

fn resolve_def<'p>(
    def: DefConstrained<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<DefValidated<'p>, TypeError> {
    let def = match def {
        DefConstrained::Fn {
            sym,
            params,
            typ,
            bdy,
        } => DefValidated::Fn {
            sym: sym.inner,
            params: params.fmap(|p| Param {
                sym: p.sym.inner,
                typ: resolve_type(p.typ),
                mutable: p.mutable,
            }),
            typ: resolve_type(typ),
            bdy: resolve_expr(bdy, uf)?,
        },
        DefConstrained::TypeDef { sym, def } => DefValidated::TypeDef {
            sym: sym.inner,
            def: resolve_typedef(def),
        },
    };

    Ok(def)
}

fn resolve_typedef<'p>(
    typedef: TypeDef<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>>,
) -> TypeDef<UniqueSym<'p>, &'p str> {
    match typedef {
        TypeDef::Struct { fields } => TypeDef::Struct {
            fields: fields
                .fmap(|(field_sym, field_typ)| (field_sym.inner, resolve_type(field_typ))),
        },
        TypeDef::Enum { .. } => todo!(),
    }
}

fn resolve_type(typ: Type<Meta<Span, UniqueSym>>) -> Type<UniqueSym> {
    match typ {
        Type::I64 => Type::I64,
        Type::U64 => Type::U64,
        Type::Bool => Type::Bool,
        Type::Unit => Type::Unit,
        Type::Never => Type::Never,
        Type::Fn { params, typ } => Type::Fn {
            params: params.fmap(resolve_type),
            typ: typ.fmap(resolve_type),
        },
        Type::Var { sym } => Type::Var { sym: sym.inner },
    }
}

/// Panic if not possible
fn partial_type_to_type<'p>(
    value: UnionIndex,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Option<Type<UniqueSym<'p>>> {
    Some(match uf.get(value).clone() {
        PartialType::I64 => Type::I64,
        PartialType::U64 => Type::U64,
        PartialType::Int => return None,
        PartialType::Bool => Type::Bool,
        PartialType::Unit => Type::Unit,
        PartialType::Never => Type::Never,
        PartialType::Var { sym } => Type::Var { sym },
        PartialType::Fn { params, typ } => Type::Fn {
            params: params
                .into_iter()
                .map(|param| partial_type_to_type(uf.find(param), uf))
                .collect::<Option<_>>()?,
            typ: Box::new(partial_type_to_type(typ, uf)?),
        },
    })
}

fn resolve_expr<'p>(
    expr: Meta<CMeta, ExprConstrained<'p>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<Meta<Type<UniqueSym<'p>>, ExprValidated<'p>>, TypeError> {
    // Type of the expression, if `None` then type is still ambiguous.
    let typ = partial_type_to_type(expr.meta.index, uf);

    let expr = match expr.inner {
        Expr::Lit { val } => {
            let val = match val {
                Lit::Int { val, .. } => match &typ {
                    None => {
                        return Err(TypeError::IntegerAmbiguous {
                            span: expr.meta.span,
                        })
                    }
                    Some(typ) => match typ {
                        Type::I64 => TLit::I64 {
                            val: val.parse().map_err(|_| TypeError::IntegerOutOfBounds {
                                span: expr.meta.span,
                                typ: "I64",
                            })?,
                        },
                        Type::U64 => TLit::U64 {
                            val: val.parse().map_err(|_| TypeError::IntegerOutOfBounds {
                                span: expr.meta.span,
                                typ: "U64",
                            })?,
                        },
                        _ => unreachable!(),
                    },
                },
                Lit::Bool { val } => TLit::Bool { val },
                Lit::Unit => TLit::Unit,
            };
            Expr::Lit { val }
        }
        Expr::Var { sym } => Expr::Var { sym: sym.inner },
        Expr::UnaryOp {
            op,
            expr: expr_inner,
        } => Expr::UnaryOp {
            op,
            expr: Box::new(resolve_expr(*expr_inner, uf)?),
        },
        Expr::BinaryOp {
            op,
            exprs: [e1, e2],
        } => Expr::BinaryOp {
            op,
            exprs: [resolve_expr(*e1, uf)?, resolve_expr(*e2, uf)?].map(Box::new),
        },
        Expr::Let {
            sym,
            mutable,
            typ,
            bnd,
            bdy,
        } => Expr::Let {
            sym: sym.inner,
            mutable,
            typ: typ.map(resolve_type),
            bnd: Box::new(resolve_expr(*bnd, uf)?),
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::If { cnd, thn, els } => Expr::If {
            cnd: Box::new(resolve_expr(*cnd, uf)?),
            thn: Box::new(resolve_expr(*thn, uf)?),
            els: Box::new(resolve_expr(*els, uf)?),
        },
        Expr::Apply { fun, args } => Expr::Apply {
            fun: Box::new(resolve_expr(*fun, uf)?),
            args: args
                .into_iter()
                .map(|arg| resolve_expr(arg, uf))
                .collect::<Result<_, _>>()?,
        },
        Expr::Loop { bdy } => Expr::Loop {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Break { bdy } => Expr::Break {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Continue => Expr::Continue,
        Expr::Return { bdy } => Expr::Return {
            bdy: Box::new(resolve_expr(*bdy, uf)?),
        },
        Expr::Seq { stmt, cnt } => Expr::Seq {
            stmt: Box::new(resolve_expr(*stmt, uf)?),
            cnt: Box::new(resolve_expr(*cnt, uf)?),
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: sym.inner,
            bnd: Box::new(resolve_expr(*bnd, uf)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: sym.inner,
            fields: fields
                .into_iter()
                .map(|(field_sym, field_bnd)| {
                    resolve_expr(field_bnd, uf).map(|bnd| (field_sym.inner, bnd))
                })
                .collect::<Result<_, _>>()?,
        },
        Expr::AccessField { strct, field } => Expr::AccessField {
            strct: Box::new(resolve_expr(*strct, uf)?),
            field: field.inner,
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    };

    Ok(Meta {
        meta: typ.unwrap(),
        inner: expr,
    })
}