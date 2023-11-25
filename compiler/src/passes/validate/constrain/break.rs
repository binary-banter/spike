use crate::passes::parse::{Constrained, Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_break<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let Some(loop_type) = env.loop_type else {
        return Err(TypeError::BreakOutsideLoop { span });
    };

    let bdy = expr::constrain_expr(bdy, env)?;
    env.uf
        .expect_equal(bdy.meta.index, loop_type, |got, expect| {
            TypeError::MismatchedLoop {
                expect,
                got,
                span_break: bdy.meta.span,
            }
        })?;

    Ok(Meta {
        meta: MetaConstrained {
            span,
            index: env.uf.add(PartialType::Never),
        },
        inner: ExprConstrained::Break { bdy: Box::new(bdy) },
    })
}
