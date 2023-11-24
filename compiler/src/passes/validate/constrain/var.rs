use crate::passes::parse::{Meta, Span};
use crate::passes::validate::{CMeta, ExprConstrained};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_var<'p>(env: &mut Env<'_, 'p>, span: Span, sym: Meta<Span, UniqueSym<'p>>) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let EnvEntry::Type { typ, .. } = env.scope[&sym.inner] else {
        return Err(TypeError::SymbolShouldBeVariable { span });
    };
    Ok(Meta {
        meta: CMeta { span, index: typ },
        inner: ExprConstrained::Var { sym },
    })
}
