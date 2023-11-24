use crate::passes::parse::{Expr, Meta, Span};
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::{access_field, apply, assign, binary_op, seq, unary_op, var};
use crate::passes::validate::constrain::lit::constrain_lit;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::constrain::uncover_globals::Env;

pub fn constrain_expr<'p>(
    expr: Meta<Span, ExprUniquified<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let span = expr.meta;

    match expr.inner {
        Expr::Lit { val } => constrain_lit(env, span, val),
        Expr::Var { sym } => var::constrain_var(env, span, sym),
        Expr::UnaryOp { op, expr } => unary_op::constrain_unary_op(env, span, op, expr),
        Expr::BinaryOp {
            op,
            exprs: [lhs, rhs],
        } => binary_op::constrain_binary_op(env, span, op, lhs, rhs),
        Expr::Let {
            sym,
            mutable,
            typ,
            bnd,
            bdy,
        } => crate::passes::validate::constrain::r#let::constrain_let(env, span, sym, mutable, typ, bnd, bdy),
        Expr::If { cnd, thn, els } => crate::passes::validate::constrain::r#if::constrain_if(env, span, cnd, thn, els),
        Expr::Apply { fun, args } => apply::constrain_apply(env, span, fun, args),
        Expr::Loop { bdy } => crate::passes::validate::constrain::r#loop::constrain_loop(env, span, bdy),
        Expr::Break { bdy } => crate::passes::validate::constrain::r#break::constrain_break(env, span, bdy),
        Expr::Continue => crate::passes::validate::constrain::r#continue::constrain_continue(env, span),
        Expr::Return { bdy } => crate::passes::validate::constrain::r#return::constrain_return(env, span, bdy),
        Expr::Seq { stmt, cnt } => seq::constrain_seq(env, span, stmt, cnt),
        Expr::Assign { sym, bnd } => assign::constrain_assign(env, span, sym, bnd),
        Expr::Struct { sym, fields } => crate::passes::validate::constrain::r#struct::constrain_struct(env, span, sym, fields),
        Expr::AccessField { strct, field } => access_field::constrain_access_field(env, span, strct, field),
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    }
}
