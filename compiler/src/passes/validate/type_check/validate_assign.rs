use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{Env, EnvEntry};
use crate::passes::validate::TExpr;
use crate::s;
use crate::utils::expect::expect;

pub fn validate_assign<'p>(
    sym: Spanned<&'p str>,
    bnd: Spanned<Expr<'p>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let entry = env.scope.get(&sym.inner).ok_or(UndeclaredVar {
        sym: (*sym.inner).to_string(),
        span: s!(sym.span),
    })?;

    let EnvEntry::Type { mutable, .. } = entry else {
        return Err(VariableShouldBeExpr {
            sym: (*sym.inner).to_string(),
        });
    };

    expect(
        *mutable,
        ModifyImmutable {
            sym: (*sym.inner).to_string(),
        },
    )?;

    let bnd = validate_expr(bnd, env)?;

    Ok(Spanned {
        span,
        inner: TExpr::Assign {
            sym: sym.inner,
            bnd: Box::new(bnd.inner),
            typ: Type::Unit,
        },
    })
}
