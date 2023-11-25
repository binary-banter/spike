use crate::passes::parse::{Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};

pub fn constrain_loop<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let loop_type = env.uf.add(PartialType::Never);

    let mut env = Env {
        uf: env.uf,
        scope: env.scope,
        loop_type: Some(loop_type),
        return_type: env.return_type,
    };

    let bdy = expr::constrain_expr(bdy, &mut env)?;

    Ok(Meta {
        meta: CMeta {
            span,
            index: loop_type,
        },
        inner: ExprConstrained::Loop { bdy: Box::new(bdy) },
    })
}
