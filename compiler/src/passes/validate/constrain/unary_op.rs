use crate::passes::parse::{Constrained, Span, Spanned, UnaryOp};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_unary_op<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    op: UnaryOp,
    expr: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let typ = match op {
        UnaryOp::Neg => PartialType::IntAmbiguous,
        UnaryOp::Not => PartialType::Bool,
    };
    let expr = expr::constrain_expr(expr, env)?;

    env.uf
        .expect_partial_type(expr.meta.index, typ, |got, expect| {
            TypeError::OperandExpect {
                expect,
                got,
                op: op.to_string(),
                span_op: span,
                span_arg: expr.meta.span,
            }
        })?;

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: expr.meta.index,
        },
        inner: ExprConstrained::UnaryOp {
            op,
            expr: Box::new(expr),
        },
    })
}
