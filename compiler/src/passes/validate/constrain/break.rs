use crate::passes::parse::{Meta, Span};
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::constrain::uncover_globals::Env;

pub fn constrain_break<'p>(env: &mut Env<'_, 'p>, span: Span, bdy: Box<Meta<Span, ExprUniquified<'p>>>) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let Some(loop_type) = env.loop_type else {
        return Err(TypeError::BreakOutsideLoop { span });
    };

    let bdy = expr::constrain_expr(*bdy, env)?;
    env.uf
        .expect_equal(bdy.meta.index, loop_type, |got, expect| {
            TypeError::TypeMismatchLoop {
                expect,
                got,
                span_break: bdy.meta.span,
            }
        })?;

    Ok(Meta {
        meta: CMeta {
            span,
            index: env.uf.add(PartialType::Never),
        },
        inner: ExprConstrained::Break { bdy: Box::new(bdy) },
    })
}
