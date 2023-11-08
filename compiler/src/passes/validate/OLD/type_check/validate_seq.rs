use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::Env;
use crate::passes::validate::TExpr;

pub fn validate_seq<'p>(
    stmt: Spanned<Expr<'p>>,
    cnt: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let stmt = validate_expr(stmt, env)?;
    let cnt = validate_expr(cnt, env)?;

    Ok(Spanned {
        span,
        inner: TExpr::Seq {
            typ: cnt.inner.typ().clone(),
            stmt: Box::new(stmt.inner),
            cnt: Box::new(cnt.inner),
        },
    })
}
