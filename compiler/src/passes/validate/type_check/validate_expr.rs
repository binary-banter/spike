use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_access_field::validate_access_field;
use crate::passes::validate::type_check::validate_apply::validate_apply;
use crate::passes::validate::type_check::validate_assign::validate_assign;
use crate::passes::validate::type_check::validate_break::validate_break;
use crate::passes::validate::type_check::validate_continue::validate_continue;
use crate::passes::validate::type_check::validate_if::validate_if;
use crate::passes::validate::type_check::validate_let::validate_let;
use crate::passes::validate::type_check::validate_lit::validate_lit;
use crate::passes::validate::type_check::validate_loop::validate_loop;
use crate::passes::validate::type_check::validate_prim::validate_prim;
use crate::passes::validate::type_check::validate_return::validate_return;
use crate::passes::validate::type_check::validate_seq::validate_seq;
use crate::passes::validate::type_check::validate_struct::validate_struct;
use crate::passes::validate::type_check::validate_switch::validate_switch;
use crate::passes::validate::type_check::validate_var::validate_var;
use crate::passes::validate::type_check::validate_variant::validate_variant;
use crate::passes::validate::type_check::Env;
use crate::passes::validate::TExpr;

#[rustfmt::skip]
pub fn validate_expr<'p>(
    expr: Spanned<Expr<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    match expr.inner {
        Expr::Lit { val } => validate_lit(val, expr.span),
        Expr::Var { sym } => validate_var(sym, expr.span, env),
        Expr::Prim { op, args } => validate_prim(op, args, expr.span, env),
        Expr::Let { sym, mutable, bnd, bdy, } => validate_let(sym, mutable, *bnd, *bdy, expr.span, env),
        Expr::If { cnd, thn, els } => validate_if(*cnd, *thn, *els, expr.span, env),
        Expr::Apply { fun, args } => validate_apply(*fun, args, expr.span, env),
        Expr::Loop { bdy } => validate_loop(*bdy, expr.span, env),
        Expr::Break { bdy } => validate_break(*bdy, expr.span, env),
        Expr::Continue => validate_continue(expr.span),
        Expr::Return { bdy } => validate_return(*bdy, expr.span, env),
        Expr::Seq { stmt, cnt } => validate_seq(*stmt, *cnt, expr.span, env),
        Expr::Assign { sym, bnd } => validate_assign(sym, *bnd, expr.span, env),
        Expr::Struct { sym, fields } => validate_struct(sym, fields, expr.span, env),
        Expr::Variant { enum_sym, variant_sym, bdy } => validate_variant(enum_sym, variant_sym, *bdy, expr.span),
        Expr::AccessField { strct, field } => validate_access_field(*strct, field, expr.span, env),
        Expr::Switch { enm, arms } => validate_switch(*enm, arms, expr.span),
    }
}
