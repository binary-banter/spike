use crate::language::lvar::{Expr, LVarProgram, Op};
use crate::type_checking::TypeError::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use miette::Diagnostic;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Integer,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer => write!(f, "Integer"),
        }
    }
}

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum TypeError {
    #[error("Variable '{sym}' was not declared yet.")]
    UndeclaredVar { sym: String },
    #[error("Operation '{op}' had incorrect arity of {arity}.")]
    IncorrectArity { op: Op, arity: usize },
    #[error("Types were mismatched. Expected '{expect}', but found '{got}'.")]
    TypeMismatch { expect: Type, got: Type },
}

pub fn type_check_program(program: &LVarProgram) -> Result<Type, TypeError> {
    type_check_expr(&program.bdy, &mut PushMap::default())
}

fn type_check_expr<'p>(
    expr: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
) -> Result<Type, TypeError> {
    match expr {
        Expr::Int { .. } => Ok(Type::Integer),
        Expr::Var { sym } => scope.get(sym).cloned().ok_or(UndeclaredVar {
            sym: (*sym).to_string(),
        }),
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
            _ => Err(IncorrectArity {
                op: *op,
                arity: args.len(),
            }),
        },
        Expr::Let { sym, bnd, bdy } => {
            type_check_expr(bnd, scope)?;
            scope.push(sym, Type::Integer, |scope| type_check_expr(bdy, scope))
        }
    }
}

fn expect_type<'p>(
    expr: &Expr<&'p str>,
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
        let program = parse_program(program).unwrap();

        if should_fail {
            assert!(type_check_program(&program).is_err());
        } else {
            assert!(type_check_program(&program).is_ok());
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as good => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/type_fail" as bad => |p| check(p, true) }
}
