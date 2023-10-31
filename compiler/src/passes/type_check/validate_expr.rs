use crate::passes::parse::{Def, Expr, Lit};
use crate::passes::parse::types::Type;
use crate::passes::type_check::check::{Env, EnvEntry};
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::{util, validate_prim, validate_struct};
use crate::passes::type_check::error::TypeError;
use crate::utils::expect::expect;

pub fn validate_expr<'p>(
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
            let entry = env.scope.get(sym).ok_or(UndeclaredVar {
                sym: (*sym).to_string(),
            })?;

            if let EnvEntry::Type { typ, .. } = entry {
                Ok(typ.clone())
            } else {
                Err(VariableShouldBeExpr {
                    sym: (*sym).to_string(),
                })
            }
        }
        Expr::Prim { op, args } => validate_prim::validate_prim(env, op, args)?,
        Expr::Let {
            sym,
            mutable,
            bnd,
            bdy,
        } => {
            let typ = validate_expr(bnd, env)?;
            env.push(
                sym,
                EnvEntry::Type {
                    mutable: *mutable,
                    typ,
                },
                |env| validate_expr(bdy, env),
            )
        }
        Expr::If { cnd, thn, els } => {
            util::expect_type(cnd, Type::Bool, env)?;
            util::expect_type_eq(thn, els, env)
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
                    util::expect_type(arg, arg_typ.clone(), env)?;
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
            let entry = env.scope.get(sym).ok_or(UndeclaredVar {
                sym: (*sym).to_string(),
            })?;

            let EnvEntry::Type { typ, mutable } = entry else {
                return Err(VariableShouldBeExpr {
                    sym: (*sym).to_string(),
                });
            };

            expect(
                *mutable,
                ModifyImmutable {
                    sym: (*sym).to_string(),
                },
            )?;

            util::expect_type(bnd, typ.clone(), env)?;
            Ok(Type::Unit)
        }
        Expr::Continue => Ok(Type::Never),
        Expr::Return { bdy } => {
            util::expect_type(bdy, env.return_type.clone(), env)?;
            Ok(Type::Never)
        }
        Expr::Struct {
            sym,
            fields: provided_fields,
        } => validate_struct::validate_struct(env, sym, provided_fields),
        Expr::AccessField { strct, field: field_sym } => {
            let typ = validate_expr(strct, env)?;
            let Type::Var { sym } = typ else {
                return Err(TypeShouldBeStruct {
                    typ: typ.clone().fmap(str::to_string),
                });
            };

            let EnvEntry::Def {
                def: Def::Struct {
                    fields: def_fields, ..
                },
            } = env.scope[sym]
            else {
                return Err(VariableShouldBeStruct {
                    sym: sym.to_string(),
                });
            };

            let Some((_, t)) = def_fields.iter().find(|&(sym, _)| sym == field_sym) else {
                return Err(UnknownStructField {
                    sym: sym.to_string(),
                });
            };

            Ok(t.clone())
        }
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    }
}
