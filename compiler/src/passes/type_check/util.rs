use crate::passes::parse::types::Type;
use crate::passes::parse::Expr;
use crate::passes::type_check::check::Env;
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::validate_expr;
use crate::utils::expect::expect;

pub fn expect_type_eq<'p>(
    e1: &Expr<&'p str>,
    e2: &Expr<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<Type<&'p str>, TypeError> {
    let t1 = validate_expr::validate_expr(e1, env)?;
    let t2 = validate_expr::validate_expr(e2, env)?;
    expect(
        t1 == t2,
        TypeMismatchEqual {
            t1: t1.clone().fmap(str::to_string),
            t2: t2.fmap(str::to_string),
        },
    )?;
    Ok(t1)
}

pub fn expect_type<'p>(
    expr: &Expr<&'p str>,
    expected: Type<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<(), TypeError> {
    let t = validate_expr::validate_expr(expr, env)?;
    expect(
        t == expected,
        TypeMismatchExpect {
            got: t.fmap(str::to_string),
            expect: expected.fmap(str::to_string),
        },
    )
}
