use crate::passes::parse::{Def, Expr, Lit, Op, PrgParsed};
use crate::passes::type_check::check::TypeError::*;
use crate::passes::type_check::PrgTypeChecked;
use crate::passes::type_check::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use miette::Diagnostic;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum TypeError {
    #[error("Variable '{sym}' was not declared yet.")]
    UndeclaredVar { sym: String },
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
    #[error("Found a break outside of a loop.")]
    BreakOutsideLoop,
    #[error("Tried to modify immutable variable 'var'")]
    ModifyImmutable { sym: String },
}

struct Env<'a, 'p> {
    scope: &'a mut PushMap<&'p str, (bool, Type)>,
    loop_type: &'a mut Option<Type>,
    in_loop: bool,
}

impl<'a, 'p> Env<'a, 'p> {
    pub fn push<O>(&mut self, k: &'p str, v: (bool, Type), sub: impl FnOnce(&mut Env<'_, 'p>) -> O) -> O {
        self.scope.push(k, v, |scope| {
            sub(&mut Env {
                scope,
                loop_type: self.loop_type,
                in_loop: self.in_loop,
            })
        })
    }

    pub fn push_iter<O>(
        &mut self,
        iterator: impl Iterator<Item = (&'p str, (bool, Type))>,
        sub: impl FnOnce(&mut Env<'_, 'p>) -> O,
    ) -> O {
        self.scope.push_iter(iterator, |scope| {
            sub(&mut Env {
                scope,
                loop_type: self.loop_type,
                in_loop: self.in_loop,
            })
        })
    }
}

impl<'p> PrgParsed<'p> {
    pub fn type_check(self) -> Result<PrgTypeChecked<'p>, TypeError> {
        let mut scope = uncover_fns(&self)?;
        let mut env = Env {
            scope: &mut scope,
            loop_type: &mut None,
            in_loop: false,
        };

        // Typecheck all definitions and collect them.
        let defs = self
            .defs
            .into_iter()
            .map(|def| match def {
                Def::Fn {
                    sym,
                    ref params,
                    ref bdy,
                    ref typ,
                } => env
                    .push_iter(params.iter().map(|p| (p.sym, (p.mutable, p.typ.clone()))), |env| {
                        expect_type(bdy, typ.clone(), env)
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

/// Returns a `PushMap` with all the functions in scope.
fn uncover_fns<'p>(program: &PrgParsed<'p>) -> Result<PushMap<&'p str, (bool, Type)>, TypeError> {
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
                    args: args.iter().map(|param| param.typ.clone()).collect(),
                };
                expect(
                    globals.insert(*sym, (false, signature)).is_none(),
                    DuplicateFunction {
                        sym: sym.to_string(),
                    },
                )?;

                let mut arg_syms = HashSet::new();
                expect(
                    args.iter().all(|param| arg_syms.insert(param.sym)),
                    DuplicateArg {
                        sym: sym.to_string(),
                    },
                )?;
            }
        }
    }

    Ok(PushMap::from(globals))
}

fn type_check_expr<'p>(expr: &Expr<&'p str>, env: &mut Env<'_, 'p>) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit { val } => match val {
            Lit::Int { .. } => Ok(Type::Int),
            Lit::Bool { .. } => Ok(Type::Bool),
            Lit::Unit => Ok(Type::Unit),
        },
        Expr::Var { sym } => env.scope.get(sym).map(|(_, t)| t).cloned().ok_or(UndeclaredVar {
            sym: sym.to_string(),
        }),
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div, [e1, e2]) => {
                expect_type(e1, Type::Int, env)?;
                expect_type(e2, Type::Int, env)?;
                Ok(Type::Int)
            }
            (Op::Minus, [e1]) => {
                expect_type(e1, Type::Int, env)?;
                Ok(Type::Int)
            }
            (Op::Read, []) => Ok(Type::Int),
            (Op::Print, [e1]) => {
                // todo: Eventually `Print` should become a function call, not an `Expr`.
                expect_type(e1, Type::Int, env)?;
                Ok(Type::Int)
            }
            (Op::GT | Op::GE | Op::LT | Op::LE, [e1, e2]) => {
                expect_type(e1, Type::Int, env)?;
                expect_type(e2, Type::Int, env)?;
                Ok(Type::Bool)
            }
            (Op::EQ | Op::NE, [e1, e2]) => {
                expect_type_eq(e1, e2, env)?;
                Ok(Type::Bool)
            }
            (Op::Not, [e1]) => {
                expect_type(e1, Type::Bool, env)?;
                Ok(Type::Bool)
            }
            (Op::LAnd | Op::LOr | Op::Xor, [e1, e2]) => {
                expect_type(e1, Type::Bool, env)?;
                expect_type(e2, Type::Bool, env)?;
                Ok(Type::Bool)
            }
            _ => panic!("Found incorrect operator during type checking"),
        },
        Expr::Let {
            sym,
            mutable,
            bnd,
            bdy,
        } => {
            let t = type_check_expr(bnd, env)?;
            env.push(sym, (*mutable, t), |env| type_check_expr(bdy, env))
        }
        Expr::If { cnd, thn, els } => {
            expect_type(cnd, Type::Bool, env)?;
            expect_type_eq(thn, els, env)
        }
        Expr::Apply { fun, args } => match type_check_expr(fun, env)? {
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
                    expect_type(arg, arg_typ.clone(), env)?;
                }

                Ok(*typ)
            }
            got => Err(TypeMismatchExpectFn { got }),
        },
        Expr::Loop { bdy } => {
            let mut loop_type = None;
            let mut env = Env {
                scope: env.scope,
                loop_type: &mut loop_type,
                in_loop: true,
            };
            type_check_expr(bdy, &mut env)?;
            Ok(loop_type.unwrap_or(Type::Never))
        }
        Expr::Break { bdy } => {
            expect(env.in_loop, BreakOutsideLoop)?;

            let bdy_type = match bdy {
                None => Type::Unit,
                Some(bdy) => type_check_expr(bdy, env)?,
            };

            if let Some(loop_type) = env.loop_type {
                expect(
                    *loop_type == bdy_type,
                    TypeMismatchEqual {
                        t1: loop_type.clone(),
                        t2: bdy_type.clone(),
                    },
                )?;
            } else {
                *env.loop_type = Some(bdy_type);
            }

            Ok(Type::Never)
        }
        Expr::Seq { stmt, cnt } => {
            type_check_expr(stmt, env)?;
            type_check_expr(cnt, env)
        },
        Expr::Assign { sym, bnd } => {
            let (mutable, typ) = env.scope.get(sym).cloned().ok_or(UndeclaredVar {
                sym: sym.to_string(),
            })?;
            expect(mutable, ModifyImmutable { sym: sym.to_string() })?;
            expect_type(bnd, typ, env)?;
            Ok(Type::Unit)
        },
    }
}

fn expect_type_eq<'p>(
    e1: &Expr<&'p str>,
    e2: &Expr<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<Type, TypeError> {
    let t1 = type_check_expr(e1, env)?;
    let t2 = type_check_expr(e2, env)?;
    expect(t1 == t2, TypeMismatchEqual { t1: t1.clone(), t2 })?;
    Ok(t1)
}

fn expect_type<'p>(
    expr: &Expr<&'p str>,
    expected: Type,
    env: &mut Env<'_, 'p>,
) -> Result<(), TypeError> {
    let t = type_check_expr(expr, env)?;
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
    use crate::passes::parse::parse::parse_program;
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
