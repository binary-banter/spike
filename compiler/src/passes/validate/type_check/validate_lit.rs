use crate::passes::parse::types::Type;
use crate::passes::parse::{Lit, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::{TExpr, TLit};
use crate::s;

pub fn validate_lit<'p>(
    val: Lit,
    span: (usize, usize),
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let inner = match val {
        Lit::Int { val } => {
            let val = val
                .parse()
                .map_err(|_| IntegerOutOfBounds { span: s!(span) })?;

            TExpr::Lit {
                val: TLit::Int { val },
                typ: Type::Int,
            }
        }
        Lit::Bool { val } => TExpr::Lit {
            val: TLit::Bool { val },
            typ: Type::Bool,
        },
        Lit::Unit => TExpr::Lit {
            val: TLit::Unit,
            typ: Type::Unit,
        },
    };

    Ok(Spanned { span, inner })
}
