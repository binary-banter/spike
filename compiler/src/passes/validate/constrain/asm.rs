use crate::passes::parse::{Constrained, Span, Spanned};
use crate::passes::select::{Instr, VarArg};
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, InstrUniquified, MetaConstrained};
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_asm<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    instrs: Vec<InstrUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    //TODO mutability checks
    //TODO check that all vars are nums

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: env.uf.add(PartialType::Unit),
        },
        inner: ExprConstrained::Asm { instrs },
    })
}
