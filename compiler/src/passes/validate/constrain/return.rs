use crate::passes::parse::{Meta, Span};
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::constrain::uncover_globals::Env;

pub fn constrain_return<'p>(env: &mut Env<'_, 'p>, span: Span, bdy: Box<Meta<Span, ExprUniquified<'p>>>) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let bdy = expr::constrain_expr(*bdy, env)?;

    env.uf
        .expect_equal(bdy.meta.index, env.return_type.inner, |bdy_typ, rtrn| {
            TypeError::MismatchedFnReturn {
                got: bdy_typ,
                expect: rtrn,
                span_got: bdy.meta.span,
                span_expected: env.return_type.meta, //TODO span of return type, should be passed via env
            }
        })?;

    Ok(Meta {
        meta: CMeta {
            span,
            index: env.uf.add(PartialType::Never),
        },
        inner: ExprConstrained::Return { bdy: Box::new(bdy) },
    })
}
