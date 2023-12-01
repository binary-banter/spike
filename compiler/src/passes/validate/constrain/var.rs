use crate::passes::parse::{Constrained, Span, Spanned};
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{ExprConstrained, MetaConstrained};
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_var<'p>(
    env: &Env<'_, 'p>,
    span: Span,
    sym: Spanned<UniqueSym<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let EnvEntry::Type { typ, .. } = env.scope[&sym.inner] else {
        return Err(TypeError::SymbolShouldBeVariable { span });
    };
    Ok(Constrained {
        meta: MetaConstrained { span, index: typ },
        inner: ExprConstrained::Var { sym },
    })
}
