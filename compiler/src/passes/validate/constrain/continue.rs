use crate::passes::parse::{Constrained, Span};
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, MetaConstrained};
use crate::utils::expect::expect;

pub fn constrain_continue<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    expect(
        env.loop_type.is_some(),
        TypeError::ContinueOutsideLoop { span },
    )?;

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: env.uf.add(PartialType::Never),
        },
        inner: ExprConstrained::Continue,
    })
}
