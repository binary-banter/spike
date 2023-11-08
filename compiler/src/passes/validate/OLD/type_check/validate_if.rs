use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{expect_type, expect_type_eq, Env};
use crate::passes::validate::TExpr;

pub fn validate_if<'p>(
    cnd: Spanned<Expr<'p>>,
    thn: Spanned<Expr<'p>>,
    els: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let cnd = validate_expr(cnd, env)?;
    let thn = validate_expr(thn, env)?;
    let els = validate_expr(els, env)?;

    expect_type(&cnd, &Type::Bool)?;
    expect_type_eq(&thn, &els)?;

    Ok(Spanned {
        span,
        inner: TExpr::If {
            typ: thn.inner.typ().clone(),
            cnd: Box::new(cnd.inner),
            thn: Box::new(thn.inner),
            els: Box::new(els.inner),
        },
    })
}
