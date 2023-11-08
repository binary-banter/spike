use crate::passes::parse::Spanned;
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::{Env, EnvEntry};
use crate::passes::validate::TExpr;
use crate::s;

pub fn validate_var<'p>(
    sym: &'p str,
    span: (usize, usize),
    env: &Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let entry = env.scope.get(&sym).ok_or(UndeclaredVar {
        sym: (*sym).to_string(),
        span: s!(span),
    })?;

    let EnvEntry::Type { typ, .. } = entry else {
        return Err(VariableShouldBeExpr {
            sym: (*sym).to_string(),
        });
    };

    Ok(Spanned {
        span,
        inner: TExpr::Var {
            sym,
            typ: typ.clone(),
        },
    })
}
