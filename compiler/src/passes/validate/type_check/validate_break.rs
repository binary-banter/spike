use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{expect_type, Env};
use crate::passes::validate::TExpr;
use crate::utils::expect::expect;

pub fn validate_break<'p>(
    bdy: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    expect(env.in_loop, BreakOutsideLoop)?;

    let bdy = validate_expr(bdy, env)?;

    if let Some(loop_type) = env.loop_type {
        expect_type(&bdy, loop_type)?;
    } else {
        *env.loop_type = Some(bdy.inner.typ().clone());
    }

    Ok(Spanned {
        span,
        inner: TExpr::Break {
            bdy: Box::new(bdy.inner),
            typ: Type::Never,
        },
    })
}
