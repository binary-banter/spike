use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, Span, Spanned, UnaryOp};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};

pub fn constrain_unary_op<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    op: UnaryOp,
    expr: Spanned<ExprUniquified<'p>>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let typ = match op {
        UnaryOp::Neg => Type::I64,
        UnaryOp::Not => Type::Bool,
    };
    let expr = expr::constrain_expr(expr, env)?;

    env.uf.expect_type(expr.meta.index, typ, |got, expect| {
        TypeError::OperandExpect {
            expect,
            got,
            op: op.to_string(),
            span_op: span,
            span_arg: expr.meta.span,
        }
    })?;

    Ok(Meta {
        meta: CMeta {
            span,
            index: expr.meta.index,
        },
        inner: ExprConstrained::UnaryOp {
            op,
            expr: Box::new(expr),
        },
    })
}
