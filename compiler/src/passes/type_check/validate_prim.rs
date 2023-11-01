use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Op};
use crate::passes::type_check::check::Env;
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::util;

pub fn validate_prim<'p>(
    env: &mut Env<'_, 'p>,
    op: &Op,
    args: &Vec<Expr<'p, &'p str>>,
) -> Result<Result<Type<&'p str>, TypeError>, TypeError> {
    Ok(match (op, args.as_slice()) {
        (Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div, [e1, e2]) => {
            util::expect_type(e1, Type::Int, env)?;
            util::expect_type(e2, Type::Int, env)?;
            Ok(Type::Int)
        }
        (Op::Minus, [e1]) => {
            util::expect_type(e1, Type::Int, env)?;
            Ok(Type::Int)
        }
        (Op::Read, []) => Ok(Type::Int),
        (Op::Print, [e1]) => {
            // todo: Eventually `Print` should become a function call, not an `Expr`.
            util::expect_type(e1, Type::Int, env)?;
            Ok(Type::Int)
        }
        (Op::GT | Op::GE | Op::LT | Op::LE, [e1, e2]) => {
            util::expect_type(e1, Type::Int, env)?;
            util::expect_type(e2, Type::Int, env)?;
            Ok(Type::Bool)
        }
        (Op::EQ | Op::NE, [e1, e2]) => {
            util::expect_type_eq(e1, e2, env)?;
            Ok(Type::Bool)
        }
        (Op::Not, [e1]) => {
            util::expect_type(e1, Type::Bool, env)?;
            Ok(Type::Bool)
        }
        (Op::LAnd | Op::LOr | Op::Xor, [e1, e2]) => {
            util::expect_type(e1, Type::Bool, env)?;
            util::expect_type(e2, Type::Bool, env)?;
            Ok(Type::Bool)
        }
        _ => panic!("Found incorrect operator during type checking"),
    })
}
