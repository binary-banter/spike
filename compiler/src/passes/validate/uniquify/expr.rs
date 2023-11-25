use crate::passes::parse::{Expr, ExprParsed, Meta, Span};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::gen_spanned_sym;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::{uniquify, ExprUniquified};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

pub fn uniquify_expr<'p>(
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
            sym: uniquify::try_get(sym, scope)?,
        },
        Expr::Assign { sym, bnd } => Expr::Assign {
            sym: uniquify::try_get(sym, scope)?,
            bnd: Box::new(uniquify_expr(*bnd, scope)?),
        },
        Expr::Struct { sym, fields } => Expr::Struct {
            sym: uniquify::try_get(sym, scope)?,
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
