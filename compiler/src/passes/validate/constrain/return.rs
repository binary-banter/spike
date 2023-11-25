use crate::passes::parse::{Constrained, Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_return<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let bdy = expr::constrain_expr(bdy, env)?;

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
        meta: MetaConstrained {
            span,
            index: env.uf.add(PartialType::Never),
        },
        inner: ExprConstrained::Return { bdy: Box::new(bdy) },
    })
}
