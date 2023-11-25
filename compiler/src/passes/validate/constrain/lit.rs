use crate::passes::parse::{Constrained, Lit, Meta, Span};
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, MetaConstrained};

pub fn constrain_lit<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    val: Lit<'p>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    // Get the type of the literal.
    let typ = match &val {
        Lit::Int { typ, .. } => {
            // If no type hint is given, use the generic `Int`.
            typ.clone().unwrap_or(PartialType::Int)
        }
        Lit::Bool { .. } => PartialType::Bool,
        Lit::Unit => PartialType::Unit,
    };

    // Add the type to the constraints.
    let index = env.uf.add(typ);

    Ok(Meta {
        meta: MetaConstrained { span, index },
        inner: ExprConstrained::Lit { val },
    })
}
