use crate::passes::parse::{Constrained, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_loop<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let loop_type = env.uf.add(PartialType::Never);

    let mut env = Env {
        uf: env.uf,
        scope: env.scope,
        loop_type: Some(loop_type),
        return_type: env.return_type,
    };

    let bdy = expr::constrain_expr(bdy, &mut env)?;

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: loop_type,
        },
        inner: ExprConstrained::Loop { bdy: Box::new(bdy) },
    })
}
