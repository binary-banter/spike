use crate::passes::parse::types::Type;
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::TExpr;
use crate::utils::expect::expect;

pub fn expect_type_eq<'p>(
    e1: &TExpr<'p, &'p str>,
    e2: &TExpr<'p, &'p str>,
) -> Result<Type<&'p str>, TypeError> {
    let t1 = e1.typ();
    let t2 = e2.typ();
    expect(
        t1 == t2,
        TypeMismatchEqual {
            t1: t1.clone().fmap(str::to_string),
            t2: t2.clone().fmap(str::to_string),
        },
    )?;
    Ok(t1.clone())
}

pub fn expect_type<'p>(
    expr: &TExpr<'p, &'p str>,
    expected: &Type<&'p str>,
) -> Result<(), TypeError> {
    let t = expr.typ();
    expect(
        t == expected,
        TypeMismatchExpect {
            got: t.clone().fmap(str::to_string),
            expect: expected.clone().fmap(str::to_string),
        },
    )
}
