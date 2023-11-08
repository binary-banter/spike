use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::TExpr;

pub fn validate_variant<'p>(
    _enum_sym: &str,
    _variant_sym: &str,
    _bdy: Spanned<Expr>,
    _span: (usize, usize),
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    todo!()
}
