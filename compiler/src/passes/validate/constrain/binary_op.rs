use crate::passes::parse::{BinaryOp, Meta, Span};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};

pub fn constrain_binary_op<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    op: BinaryOp,
    lhs: Box<Meta<Span, ExprUniquified<'p>>>,
    rhs: Box<Meta<Span, ExprUniquified<'p>>>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    // input: None = Any but equal, Some = expect this
    // output: None = Same as input, Some = this
    let (input, output) = match op {
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            (Some(PartialType::Int), None)
        }
        BinaryOp::LAnd | BinaryOp::LOr | BinaryOp::Xor => (Some(PartialType::Bool), None),
        BinaryOp::GT | BinaryOp::GE | BinaryOp::LE | BinaryOp::LT => {
            (Some(PartialType::Int), Some(PartialType::Bool))
        }
        BinaryOp::EQ | BinaryOp::NE => (None, Some(PartialType::Bool)),
    };

    let e1 = expr::constrain_expr(*lhs, env)?;
    let e2 = expr::constrain_expr(*rhs, env)?;

    // Check inputs satisfy constraints
    if let Some(input) = input {
        let mut check = |expr: &Meta<CMeta, ExprConstrained<'p>>| {
            env.uf
                .expect_partial_type(expr.meta.index, input.clone(), |got, expect| {
                    TypeError::OperandExpect {
                        expect,
                        got,
                        op: op.to_string(),
                        span_op: span,
                        span_arg: expr.meta.span,
                    }
                })
        };

        check(&e1)?;
        check(&e2)?;
    }

    // Check inputs equal
    let input_index = env
        .uf
        .expect_equal(e1.meta.index, e2.meta.index, |lhs, rhs| {
            TypeError::OperandEqual {
                lhs,
                rhs,
                op: op.to_string(),
                span_op: span,
                span_lhs: e1.meta.span,
                span_rhs: e2.meta.span,
            }
        })?;

    // Generate output index
    let output_index = match output {
        None => input_index,
        Some(e) => env.uf.add(e),
    };

    Ok(Meta {
        meta: CMeta {
            span,
            index: output_index,
        },
        inner: ExprConstrained::BinaryOp {
            op,
            exprs: [e1, e2].map(Box::new),
        },
    })
}
