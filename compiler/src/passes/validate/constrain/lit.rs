use crate::passes::parse::types::IntType;
use crate::passes::parse::{Constrained, Lit, Span};
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, MetaConstrained};

pub fn constrain_lit<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    val: Lit<&'p str>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    // Get the type of the literal.
    let typ = match &val {
        Lit::Int(val) => match val.rfind(&['i', 'u']) {
            Some(suffix) => {
                match &val[suffix..] {
                    "i8" => PartialType::Int(IntType::I8),
                    "u8" => PartialType::Int(IntType::U8),
                    "i16" => PartialType::Int(IntType::I16),
                    "u16" => PartialType::Int(IntType::U16),
                    "i32" => PartialType::Int(IntType::I32),
                    "u32" => PartialType::Int(IntType::U32),
                    "i64" => PartialType::Int(IntType::I64),
                    "u64" => PartialType::Int(IntType::U64),
                    _ => PartialType::IntAmbiguous,
                }
            }
            None => PartialType::IntAmbiguous,
        },
        Lit::Bool { .. } => PartialType::Bool,
        Lit::Unit => PartialType::Unit,
    };

    // Add the type to the constraints.
    let index = env.uf.add(typ);

    Ok(Constrained {
        meta: MetaConstrained { span, index },
        inner: ExprConstrained::Lit { val },
    })
}
