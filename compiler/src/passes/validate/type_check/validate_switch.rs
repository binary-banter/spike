use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::TExpr;

pub fn validate_switch<'p>(
    _enm: Spanned<Expr>,
    _arms: Vec<(&str, &str, Box<Spanned<Expr>>)>,
    _span: (usize, usize),
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    todo!()
}
