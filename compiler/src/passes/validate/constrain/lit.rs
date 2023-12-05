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
                let int = match &val[suffix..] {
                    "i8" => IntType::I64,
                    "u8" => IntType::U64,
                    "i16" => IntType::I64,
                    "u16" => IntType::U64,
                    "i32" => IntType::I64,
                    "u32" => IntType::U64,
                    "i64" => IntType::I64,
                    "u64" => IntType::U64,
                    _ => unreachable!(),
                };
                PartialType::Int(int)
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
