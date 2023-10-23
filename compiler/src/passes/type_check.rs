use crate::language::lvar::{Def, Expr, Lit, Op, PrgParsed, PrgTypeChecked};
use crate::passes::type_check::TypeError::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use itertools::Itertools;
use miette::Diagnostic;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Fn { typ: Box<Type>, args: Vec<Type> },
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Fn { typ, args } => write!(f, "fn({}) -> {}", args.iter().format(", "), typ),
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
    #[error("Types were mismatched. Expected function, but found '{got}'.")]
    TypeMismatchExpectFn { got: Type },
    #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    TypeMismatchEqual { t1: Type, t2: Type },
    #[error("There are multiple functions named `{sym}`.")]
    DuplicateFunction { sym: String },
    #[error("Function `{sym}` has duplicate argument names.")]
    DuplicateArg { sym: String },
    #[error("Function `{expected}` has {expected} arguments, but found {got} arguments.")]
    ArgCountMismatch { expected: usize, got: usize },
    #[error("The program doesn't have a main function.")]
    NoMain,
}

impl<'p> PrgParsed<'p> {
    pub fn type_check(self) -> Result<PrgTypeChecked<'p>, TypeError> {
        let mut scope = uncover_fns(&self)?;

        let defs = self
            .defs
            .into_iter()
            .map(|def| match def {
                Def::Fn {
                    sym,
                    ref params,
                    ref bdy,
                    ref typ,
                } => scope
                    .push_iter(params.iter().cloned(), |scope| {
                        expect_type(bdy, scope, typ.clone())
                    })
                    .map(|_| (sym, def)),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        expect(defs.contains_key("main"), NoMain)?;

        Ok(PrgTypeChecked {
            defs,
            entry: self.entry,
        })
    }
}

fn uncover_fns<'p>(program: &PrgParsed<'p>) -> Result<PushMap<&'p str, Type>, TypeError> {
    let mut globals = HashMap::new();

    for def in &program.defs {
        match def {
            Def::Fn {
                sym,
                params: args,
                typ,
                ..
            } => {
                let signature = Type::Fn {
                    typ: Box::new(typ.clone()),
                    args: args.iter().map(|(_, t)| t.clone()).collect(),
                };
                expect(
                    globals.insert(*sym, signature).is_none(),
                    DuplicateFunction {
                        sym: sym.to_string(),
                    },
                )?;

                let mut arg_syms = HashSet::new();
                expect(
                    args.iter().all(|(sym, _)| arg_syms.insert(sym)),
                    DuplicateArg {
                        sym: sym.to_string(),
                    },
                )?;
            }
        }
    }

    Ok(PushMap::from(globals))
}

fn type_check_expr<'p>(
    expr: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit {
            val: Lit::Bool { .. },
        } => Ok(Type::Bool),
        Expr::Lit {
            val: Lit::Int { .. },
        } => Ok(Type::Int),
        Expr::Var { sym } => scope.get(sym).cloned().ok_or(UndeclaredVar {
            sym: (*sym).to_string(),
        }),
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div, [e1, e2]) => {
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
            (Op::GT | Op::GE | Op::LT | Op::LE, [e1, e2]) => {
                expect_type(e1, scope, Type::Int)?;
                expect_type(e2, scope, Type::Int)?;
                Ok(Type::Bool)
            }
            (Op::EQ | Op::NE, [e1, e2]) => {
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
        Expr::Apply { fun, args } => match type_check_expr(fun, scope)? {
            Type::Fn {
                typ,
                args: expected_types,
            } => {
                if expected_types.len() != args.len() {
                    return Err(ArgCountMismatch {
                        expected: expected_types.len(),
                        got: args.len(),
                    });
                }

                for (arg, arg_typ) in args.iter().zip(expected_types.iter()) {
                    expect_type(arg, scope, arg_typ.clone())?;
                }

                Ok(*typ)
            }
            got => Err(TypeMismatchExpectFn { got }),
        },
    }
}

fn expect_type_eq<'p>(
    e1: &Expr<&'p str>,
    e2: &Expr<&'p str>,
    scope: &mut PushMap<&'p str, Type>,
) -> Result<Type, TypeError> {
    let t1 = type_check_expr(e1, scope)?;
    let t2 = type_check_expr(e2, scope)?;
    expect(t1 == t2, TypeMismatchEqual { t1: t1.clone(), t2 })?;
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
            got: t,
            expect: expected,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::passes::parse::parse_program;
    use test_each_file::test_each_file;

    fn check([test]: [&str; 1], should_fail: bool) {
        let mut test = test.split('#');
        let program = test.nth(3).unwrap().trim();
        let program = parse_program(program).unwrap();
        let res = program.type_check();

        match (res, should_fail) {
            (Ok(_), true) => panic!("Program should not pass type-checking."),
            (Err(e), false) => {
                panic!("Program should have passed type-checking, but returned error: '{e}'.")
            }
            _ => {}
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as type_check_succeed => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/fail/type_check" as type_check_fail => |p| check(p, true) }
}
