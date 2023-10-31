use crate::passes::parse::{Def, Expr, Lit, Op, PrgParsed};
use crate::passes::type_check::check::TypeError::*;
use crate::passes::type_check::{PrgTypeChecked, Type};
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
    TypeMismatchExpect {
        expect: Type<String>,
        got: Type<String>,
    },
    #[error("Types were mismatched. Expected function, but found '{got}'.")]
    TypeMismatchExpectFn { got: Type<String> },
    #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    TypeMismatchEqual { t1: Type<String>, t2: Type<String> },
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
    #[error("Tried to modify immutable variable '{sym}'")]
    ModifyImmutable { sym: String },
    #[error("The variable {sym} refers to a definition.'")]
    VariableRefersToDef { sym: String },
}

struct Env<'a, 'p> {
    scope: &'a mut PushMap<&'p str, EnvEntry<'p>>,
    loop_type: &'a mut Option<Type<&'p str>>,
    in_loop: bool,
    return_type: &'a Type<&'p str>,
}

enum EnvEntry<'p> {
    Type{
        mutable: bool,
        typ: Type<&'p str>,
    },
    Def{
        def: &'p Def<&'p str, Expr<&'p str>>
    },
}

impl<'a, 'p> Env<'a, 'p> {
    pub fn push<O>(
        &mut self,
        k: &'p str,
        v: EnvEntry<'p>,
        sub: impl FnOnce(&mut Env<'_, 'p>) -> O,
    ) -> O {
        self.scope.push(k, v, |scope| {
            sub(&mut Env {
                scope,
                loop_type: self.loop_type,
                in_loop: self.in_loop,
                return_type: self.return_type,
            })
        })
    }
}

impl<'p> PrgParsed<'p> {
    pub fn type_check(self) -> Result<PrgTypeChecked<'p>, TypeError> {
        let mut scope = uncover_globals(&self)?;

        // Typecheck all definitions and collect them.
        self
            .defs
            .iter()
            .map(|def| match def {
                Def::Fn {
                    sym,
                    ref params,
                    ref bdy,
                    ref typ,
                } => scope
                    .push_iter(
                        params.iter().map(|p| (p.sym, EnvEntry::Type { mutable: p.mutable, typ: p.typ.clone()})),
                        |scope| {
                            let mut env = Env {
                                scope,
                                loop_type: &mut None,
                                in_loop: false,
                                return_type: typ,
                            };

                            expect_type(bdy, typ.clone(), &mut env)
                        },
                    ),
                Def::Struct { fields: types, .. } | Def::Enum { variants: types, .. } => {
                    for (_, typ) in types {
                        validate_type(typ, &scope)?;
                    }
                    Ok(())
                },
            })
            .collect::<Result<(), _>>()?;

        let defs = self.defs.into_iter().map(|def| (*def.sym(), def)).collect::<HashMap<_, _>>();

        expect(defs.contains_key("main"), NoMain)?;

        Ok(PrgTypeChecked {
            defs,
            entry: self.entry,
        })
    }
}

/// Verifies that the given type exists in the current scope.
fn validate_type<'p>(typ: &'p Type<&'p str>, scope: &PushMap<&str, EnvEntry<'p>>) -> Result<(), TypeError> {
    match typ {
        Type::Int | Type::Bool | Type::Unit | Type::Never=> {}
        Type::Fn { typ, params } => {
            validate_type(typ, scope)?;

            for typ in params {
                validate_type(typ, scope)?;
            }
        }
        Type::Var { sym } => {
            expect(scope.contains(sym), UndeclaredVar {
                sym: sym.to_string(),
            })?;
        }
    }

    Ok(())
}

/// Returns a `PushMap` with all the functions in scope.
fn uncover_globals<'p>(
    program: &'p PrgParsed<'p>,
) -> Result<PushMap<&'p str, EnvEntry<'p>>, TypeError> {
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
                    params: args.iter().map(|param| param.typ.clone()).collect(),
                };
                expect(
                    globals.insert(*sym, EnvEntry::Type{ mutable: false, typ: signature}).is_none(),
                    DuplicateFunction {
                        sym: (*sym).to_string(),
                    },
                )?;

                let mut arg_syms = HashSet::new();
                expect(
                    args.iter().all(|param| arg_syms.insert(param.sym)),
                    DuplicateArg {
                        sym: (*sym).to_string(),
                    },
                )?;
            }
            def @ (Def::Struct { sym, .. } | Def::Enum { sym, .. }) => {
                globals.insert(sym, EnvEntry::Def { def } );
            },
        }
    }

    Ok(PushMap::from(globals))
}

fn validate_expr<'p>(
    expr: &Expr<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<Type<&'p str>, TypeError> {
    match expr {
        Expr::Lit { val } => match val {
            Lit::Int { .. } => Ok(Type::Int),
            Lit::Bool { .. } => Ok(Type::Bool),
            Lit::Unit => Ok(Type::Unit),
        },
        Expr::Var { sym } => {
            let entry = env.scope.get(sym).ok_or(UndeclaredVar { sym: (*sym).to_string() })?;

            if let EnvEntry::Type { typ, .. } = entry{
                Ok(typ.clone())
            } else {
                Err(VariableRefersToDef { sym: (*sym).to_string() })
            }
        },
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
            let typ = validate_expr(bnd, env)?;
            env.push(sym, EnvEntry::Type{mutable: *mutable, typ}, |env| validate_expr(bdy, env))
        }
        Expr::If { cnd, thn, els } => {
            expect_type(cnd, Type::Bool, env)?;
            expect_type_eq(thn, els, env)
        }
        Expr::Apply { fun, args } => match validate_expr(fun, env)? {
            Type::Fn {
                typ,
                params: expected_types,
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
            got => Err(TypeMismatchExpectFn {
                got: got.fmap(str::to_string),
            }),
        },
        Expr::Loop { bdy } => {
            let mut loop_type = None;
            let mut env = Env {
                scope: env.scope,
                loop_type: &mut loop_type,
                in_loop: true,
                return_type: env.return_type,
            };
            validate_expr(bdy, &mut env)?;
            Ok(loop_type.unwrap_or(Type::Never))
        }
        Expr::Break { bdy } => {
            expect(env.in_loop, BreakOutsideLoop)?;

            let bdy_type = validate_expr(bdy, env)?;

            if let Some(loop_type) = env.loop_type {
                expect(
                    *loop_type == bdy_type,
                    TypeMismatchEqual {
                        t1: loop_type.clone().fmap(str::to_string),
                        t2: bdy_type.fmap(str::to_string),
                    },
                )?;
            } else {
                *env.loop_type = Some(bdy_type);
            }

            Ok(Type::Never)
        }
        Expr::Seq { stmt, cnt } => {
            validate_expr(stmt, env)?;
            validate_expr(cnt, env)
        }
        Expr::Assign { sym, bnd } => {
            let entry = env.scope.get(sym).ok_or(UndeclaredVar { sym: (*sym).to_string() })?;

            let EnvEntry::Type { typ, mutable } = entry else {
                return Err(VariableRefersToDef { sym: (*sym).to_string() })
            };

            expect(
                *mutable,
                ModifyImmutable { sym: (*sym).to_string() },
            )?;

            expect_type(bnd, typ.clone(), env)?;
            Ok(Type::Unit)
        }
        Expr::Continue => Ok(Type::Never),
        Expr::Return { bdy } => {
            expect_type(bdy, env.return_type.clone(), env)?;
            Ok(Type::Never)
        }
        Expr::Struct { .. } => todo!(),
        Expr::Variant { .. } => todo!(),
        Expr::AccessField { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    }
}

fn expect_type_eq<'p>(
    e1: &Expr<&'p str>,
    e2: &Expr<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<Type<&'p str>, TypeError> {
    let t1 = validate_expr(e1, env)?;
    let t2 = validate_expr(e2, env)?;
    expect(
        t1 == t2,
        TypeMismatchEqual {
            t1: t1.clone().fmap(str::to_string),
            t2: t2.fmap(str::to_string),
        },
    )?;
    Ok(t1)
}

fn expect_type<'p>(
    expr: &Expr<&'p str>,
    expected: Type<&'p str>,
    env: &mut Env<'_, 'p>,
) -> Result<(), TypeError> {
    let t = validate_expr(expr, env)?;
    expect(
        t == expected,
        TypeMismatchExpect {
            got: t.fmap(str::to_string),
            expect: expected.fmap(str::to_string),
        },
    )
}
