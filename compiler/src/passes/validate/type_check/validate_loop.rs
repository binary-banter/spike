use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::Env;
use crate::passes::validate::TExpr;

pub fn validate_loop<'p>(
    bdy: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let mut loop_type = None;
    let mut env = Env {
        scope: env.scope,
        loop_type: &mut loop_type,
        in_loop: true,
        return_type: env.return_type,
    };
    let bdy = validate_expr(bdy, &mut env)?;

    Ok(Spanned {
        span,
        inner: TExpr::Loop {
            bdy: Box::new(bdy.inner),
            typ: loop_type.unwrap_or(Type::Never),
        },
    })
}
