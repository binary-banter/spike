use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{Env, EnvEntry};
use crate::passes::validate::TExpr;

pub fn validate_let<'p>(
    sym: &'p str,
    mutable: bool,
    bnd: Spanned<Expr<'p>>,
    bdy: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let bnd = validate_expr(bnd, env)?;
    let bdy = env.push(
        sym,
        EnvEntry::Type {
            mutable,
            typ: bnd.inner.typ().clone(),
        },
        |env| validate_expr(bdy, env),
    )?;

    Ok(Spanned {
        span,
        inner: TExpr::Let {
            typ: bdy.inner.typ().clone(),
            sym,
            bnd: Box::new(bnd.inner),
            bdy: Box::new(bdy.inner),
        },
    })
}
