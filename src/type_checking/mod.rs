use crate::language::lvar::{Expr, LVarProgram, Op};
use crate::type_checking::TypeError::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
}

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Variable was not declared yet.")]
    UndeclaredVar,
    #[error("Prim Op had incorrect arity.")]
    IncorrectArity,
    #[error("Types were mismatched.")]
    TypeMismatch { expect: Type, got: Type },
}

pub fn type_check_program(program: &LVarProgram) -> Result<Type, TypeError> {
    type_check_expr(&program.bdy, &mut PushMap::default())
}

fn type_check_expr<'p>(expr: &Expr< &'p str>, scope: &mut PushMap<&'p str, Type>) -> Result<Type, TypeError> {
    match expr {
        Expr::Int { .. } => Ok(Type::Integer),
        Expr::Var { sym } => scope.get(sym).cloned().ok_or(UndeclaredVar),
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus | Op::Minus, [e1, e2]) => {
                expect_type(e1, scope, Type::Integer)?;
                expect_type(e2, scope, Type::Integer)?;
                Ok(Type::Integer)
            }
            (Op::Minus, [e1]) => {
                expect_type(e1, scope, Type::Integer)?;
                Ok(Type::Integer)
            }
            (Op::Read, []) => Ok(Type::Integer),
            (Op::Print, [e1]) => {
                expect_type(e1, scope, Type::Integer)?;
                Ok(Type::Integer)
            }
            _ => Err(IncorrectArity),
        },
        Expr::Let { sym, bnd, bdy } => {
            type_check_expr(bnd, scope)?;
            scope.push(sym.clone(), Type::Integer, |scope| {
                type_check_expr(bdy, scope)
            })
        }
    }
}

fn expect_type<'p>(
    expr: &Expr< &'p str>,
    scope: &mut PushMap<&'p str, Type>,
    expected: Type,
) -> Result<(), TypeError> {
    let t = type_check_expr(expr, scope)?;
    expect(
        matches!(t, Type::Integer),
        TypeMismatch {
            got: t.clone(),
            expect: expected,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_program;
    use crate::type_checking::type_check_program;
    use test_each_file::test_each_file;

    fn check([test]: [&str; 1], should_fail: bool) {
        let mut test = test.split('#');
        let program = test.nth(3).unwrap().trim();
        let program = parse_program(program).unwrap().1;

        if should_fail {
            assert!(type_check_program(&program).is_err());
        } else {
            assert!(type_check_program(&program).is_ok());
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as good => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/type_fail" as bad => |p| check(p, true) }
}
