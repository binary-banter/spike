use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{expect_type, Env};
use crate::passes::validate::TExpr;
use crate::utils::expect::expect;
use functor_derive::Functor;

pub fn validate_apply<'p>(
    fun: Spanned<Expr<'p>>,
    args: Vec<Spanned<Expr<'p>>>,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let fun = validate_expr(fun, env)?;
    let args = args
        .into_iter()
        .map(|arg| validate_expr(arg, env))
        .collect::<Result<Vec<_>, _>>()?;

    let Type::Fn { params, typ } = fun.inner.typ() else {
        return Err(TypeMismatchExpectFn {
            got: fun.inner.typ().clone().fmap(str::to_string),
        });
    };

    expect(
        params.len() == args.len(),
        ArgCountMismatch {
            expected: params.len(),
            got: args.len(),
        },
    )?;

    for (arg, param_type) in args.iter().zip(params.iter()) {
        expect_type(arg, param_type)?;
    }

    Ok(Spanned {
        span,
        inner: TExpr::Apply {
            typ: (**typ).clone(),
            fun: Box::new(fun.inner),
            args: args.fmap(|arg| arg.inner),
        },
    })
}
