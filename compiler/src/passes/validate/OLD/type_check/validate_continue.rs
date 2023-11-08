use crate::passes::parse::types::Type;
use crate::passes::parse::Spanned;
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::TExpr;

pub fn validate_continue<'p>(
    span: (usize, usize),
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    Ok(Spanned {
        span,
        inner: TExpr::Continue { typ: Type::Never },
    })
}
