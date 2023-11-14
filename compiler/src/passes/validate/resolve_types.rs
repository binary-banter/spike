use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Lit, Meta, Param, Span, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::generate_constraints::PartialType;
use crate::passes::validate::{
    CMeta, DefConstrained, DefValidated, ExprConstrained, ExprValidated, PrgConstrained,
    PrgValidated, TLit,
};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::UnionFind;
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

fn resolve_expr<'p>(
    expr: Meta<CMeta, ExprConstrained<'p>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<Meta<Type<UniqueSym<'p>>, ExprValidated<'p>>, TypeError> {
    todo!()
    // let (typ, expr) = match expr.inner {
    //     ExprConstrained::Lit { val } => {
    //         let (typ, val) = match val {
    //             Lit::Int { val } => {
    //                 match &uf.get(expr.meta.index) {
    //                     PartialType::I64 => {
    //                         let val = val.parse().map_err(|_| TypeError::IntegerOutOfBounds {
    //                             span: expr.meta.span,
    //                             typ: "I64",
    //                         })?;
    //                         (Type::I64, TLit::I64 { val })
    //                     },
    //                     PartialType::U64 => todo!(),
    //                     PartialType::Int => {
    //                         return Err(dbg!(TypeError::IntegerAmbiguous {
    //                             span: expr.meta.span
    //                         }))
    //                     },
    //                     _ => unreachable!(),
    //                 }
    //             },
    //             Lit::Bool { val } => (Type::Bool ,TLit::Bool { val }),
    //             Lit::Unit => (Type::Unit, TLit::Unit),
    //         };
    //
    //         (typ, Expr::Lit { val })
    //     },
    //     ExprConstrained::Var { .. } => todo!(),
    //     ExprConstrained::Prim { .. } => todo!(),
    //     ExprConstrained::Let { .. } => todo!(),
    //     ExprConstrained::If { .. } => todo!(),
    //     ExprConstrained::Apply { .. } => todo!(),
    //     ExprConstrained::Loop { .. } => todo!(),
    //     ExprConstrained::Break { .. } => todo!(),
    //     ExprConstrained::Continue => todo!(),
    //     ExprConstrained::Return { .. } => todo!(),
    //     ExprConstrained::Seq { .. } => todo!(),
    //     ExprConstrained::Assign { .. } => todo!(),
    //     ExprConstrained::Struct { .. } => todo!(),
    //     ExprConstrained::Variant { .. } => todo!(),
    //     ExprConstrained::AccessField { .. } => todo!(),
    //     ExprConstrained::Switch { .. } => todo!(),
    // };
    //
    // Ok(Meta {
    //     meta: typ,
    //     inner: expr,
    // })
}
