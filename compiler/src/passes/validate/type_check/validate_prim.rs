use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Op, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{expect_type, expect_type_eq, Env};
use crate::passes::validate::TExpr;
use functor_derive::Functor;

pub fn validate_prim<'p>(
    env: &mut Env<'_, 'p>,
    op: Op,
    args: Vec<Spanned<Expr<'p>>>,
) -> Result<TExpr<'p, &'p str>, TypeError> {
    let args = args
        .into_iter()
        .map(|arg| validate_expr(arg, env))
        .collect::<Result<Vec<_>, _>>()?;

    let typ = match &(op, args.as_slice()) {
        (Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div, [e1, e2]) => {
            expect_type(e1, &Type::Int)?;
            expect_type(e2, &Type::Int)?;
            Type::Int
        }
        (Op::Minus, [e1]) => {
            expect_type(e1, &Type::Int)?;
            Type::Int
        }
        (Op::Read, []) => Type::Int,
        (Op::Print, [e1]) => {
            // todo: Eventually `Print` should become a function call, not an `Expr`.
            expect_type(e1, &Type::Int)?;
            Type::Int
        }
        (Op::GT | Op::GE | Op::LT | Op::LE, [e1, e2]) => {
            expect_type(e1, &Type::Int)?;
            expect_type(e2, &Type::Int)?;
            Type::Bool
        }
        (Op::EQ | Op::NE, [e1, e2]) => {
            expect_type_eq(e1, e2)?;
            Type::Bool
        }
        (Op::Not, [e1]) => {
            expect_type(e1, &Type::Bool)?;
            Type::Bool
        }
        (Op::LAnd | Op::LOr | Op::Xor, [e1, e2]) => {
            expect_type(e1, &Type::Bool)?;
            expect_type(e2, &Type::Bool)?;
            Type::Bool
        }
        _ => panic!("Found incorrect operator during type checking"),
    };

    Ok(TExpr::Prim {
        op,
        args: args.fmap(|arg| arg.inner),
        typ,
    })
}
