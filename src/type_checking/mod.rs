use crate::interpreter::value::Val;
use crate::language::lvar::{Expr, LVarProgram, Op};
use crate::type_checking::TypeError::{
    IncorrectArity, TypeMismatchEqual, TypeMismatchExpect, UndeclaredVar,
};
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use miette::Diagnostic;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
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
    TypeMismatchExpect { expect: Type, got: Type },
    #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    TypeMismatchEqual { t1: Type, t2: Type },
}

pub fn type_check_program(program: &LVarProgram) -> Result<Type, TypeError> {
    type_check_expr(&program.bdy, &mut PushMap::default())
}

fn type_check_expr<'p>(
    expr: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
) -> Result<Type, TypeError> {
    match expr {
        Expr::Val {
            val: Val::Bool { .. },
        } => Ok(Type::Bool),
        Expr::Val {
            val: Val::Int { .. },
        } => Ok(Type::Int),
        Expr::Var { sym } => scope.get(sym).cloned().ok_or(UndeclaredVar {
            sym: (*sym).to_string(),
        }),
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus | Op::Minus, [e1, e2]) => {
                expect_type(e1, scope, Type::Int)?;
                expect_type(e2, scope, Type::Int)?;
                Ok(Type::Int)
            }
            (Op::Minus, [e1]) => {
                expect_type(e1, scope, Type::Int)?;
                Ok(Type::Int)
            }
            (Op::Read, []) => Ok(Type::Int),
            (Op::Print, [e1]) => {
                // todo: Eventually `Print` should become a function call, not an `Expr`.
                // type_check_expr(e1, scope)
                expect_type(e1, scope, Type::Int)?;
                Ok(Type::Int)
            }
            (Op::Greater | Op::GreaterOrEqual | Op::Less | Op::LessOrEqual, [e1, e2]) => {
                expect_type(e1, scope, Type::Int)?;
                expect_type(e2, scope, Type::Int)?;
                Ok(Type::Bool)
            }
            (Op::Equal | Op::NotEqual, [e1, e2]) => {
                expect_type_eq(e1, e2, scope)?;
                Ok(Type::Bool)
            }
            (Op::Not, [e1]) => {
                expect_type(e1, scope, Type::Bool)?;
                Ok(Type::Bool)
            }
            (Op::LAnd | Op::LOr | Op::Xor, [e1, e2]) => {
                expect_type(e1, scope, Type::Bool)?;
                expect_type(e2, scope, Type::Bool)?;
                Ok(Type::Bool)
            }
            _ => Err(IncorrectArity {
                op: *op,
                arity: args.len(),
            }),
        },
        Expr::Let { sym, bnd, bdy } => {
            let t = type_check_expr(bnd, scope)?;
            scope.push(sym, t, |scope| type_check_expr(bdy, scope))
        }
        Expr::If { cnd, thn, els } => {
            expect_type(cnd, scope, Type::Bool)?;
            expect_type_eq(thn, els, scope)
        }
    }
}

fn expect_type_eq<'p>(
    e1: &Expr<&'p str>,
    e2: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
) -> Result<Type, TypeError> {
    let t1 = type_check_expr(e1, scope)?;
    let t2 = type_check_expr(e2, scope)?;
    expect(t1 == t2, TypeMismatchEqual { t1, t2 })?;
    Ok(t1)
}

fn expect_type<'p>(
    expr: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
    expected: Type,
) -> Result<(), TypeError> {
    let t = type_check_expr(expr, scope)?;
    expect(
        t == expected,
        TypeMismatchExpect {
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
