use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{expect_type, Env};
use crate::passes::validate::TExpr;

pub fn validate_return<'p>(
    bdy: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let bdy = validate_expr(bdy, env)?;

    expect_type(&bdy, env.return_type)?;

    Ok(Spanned {
        span,
        inner: TExpr::Return {
            bdy: Box::new(bdy.inner),
            typ: Type::Never,
        },
    })
}
