use crate::passes::parse::{Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};

pub fn constrain_seq<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    stmt: Spanned<ExprUniquified<'p>>,
    cnt: Spanned<ExprUniquified<'p>>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let stmt = expr::constrain_expr(stmt, env)?;
    let cnt = expr::constrain_expr(cnt, env)?;

    Ok(Meta {
        meta: CMeta {
            span,
            index: cnt.meta.index,
        },
        inner: ExprConstrained::Seq {
            stmt: Box::new(stmt),
            cnt: Box::new(cnt),
        },
    })
}
