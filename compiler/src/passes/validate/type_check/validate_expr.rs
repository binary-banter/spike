use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Lit, Spanned, TypeDef};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::validate_prim::validate_prim;
use crate::passes::validate::type_check::validate_struct::validate_struct;
use crate::passes::validate::type_check::{expect_type, expect_type_eq, Env, EnvEntry};
use crate::passes::validate::TExpr;
use crate::utils::expect::expect;

pub fn validate_expr<'p>(
    expr: Spanned<Expr<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<TExpr<'p, &'p str>, TypeError> {
    let span = (expr.span.0, expr.span.1 - expr.span.0);

    Ok(match expr.expr {
        Expr::Lit { val } => match val {
            Lit::Int { val } => {
                // todo: fix this mess by passing in a string!
                let val = i32::try_from(val).map_err(|_| IntegerOutOfBounds { span })?;
                let val = val as i64;
                TExpr::Lit { val: Lit::Int { val }, typ: Type::Int }
            },
            val @ Lit::Bool { .. } => TExpr::Lit { val, typ: Type::Bool },
            val @ Lit::Unit => TExpr::Lit { val, typ : Type::Unit },
        },
        Expr::Var { sym, .. } => {
            let entry = env.scope.get(&sym).ok_or(UndeclaredVar {
                sym: (*sym).to_string(),
            })?;

            if let EnvEntry::Type { typ, .. } = entry {
                TExpr::Var {
                    sym,
                    typ: typ.clone(),
                }
            } else {
                return Err(VariableShouldBeExpr {
                    sym: (*sym).to_string(),
                });
            }
        }
        Expr::Prim { op, args, .. } => validate_prim(env, op, args)?,
        Expr::Let {
            sym,
            mutable,
            bnd,
            bdy,
            ..
        } => {
            let bnd = validate_expr(*bnd, env)?;
            let bdy = env.push(
                sym,
                EnvEntry::Type {
                    mutable,
                    typ: bnd.typ().clone(),
                },
                |env| validate_expr(*bdy, env),
            )?;

            TExpr::Let {
                typ: bdy.typ().clone(),
                sym,
                bnd: Box::new(bnd),
                bdy: Box::new(bdy),
            }
        }
        Expr::If { cnd, thn, els, .. } => {
            let cnd = validate_expr(*cnd, env)?;
            let thn = validate_expr(*thn, env)?;
            let els = validate_expr(*els, env)?;

            expect_type(&cnd, &Type::Bool)?;
            expect_type_eq(&thn, &els)?;

            TExpr::If {
                typ: thn.typ().clone(),
                cnd: Box::new(cnd),
                thn: Box::new(thn),
                els: Box::new(els),
            }
        }
        Expr::Apply { fun, args, .. } => {
            let fun = validate_expr(*fun, env)?;
            let args = args
                .into_iter()
                .map(|arg| validate_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;

            let Type::Fn { params, typ } = fun.typ() else {
                return Err(TypeMismatchExpectFn {
                    got: fun.typ().clone().fmap(str::to_string),
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

            TExpr::Apply {
                typ: (**typ).clone(),
                fun: Box::new(fun),
                args,
            }
        }
        Expr::Loop { bdy, .. } => {
            let mut loop_type = None;
            let mut env = Env {
                scope: env.scope,
                loop_type: &mut loop_type,
                in_loop: true,
                return_type: env.return_type,
            };
            let bdy = validate_expr(*bdy, &mut env)?;
            TExpr::Loop {
                bdy: Box::new(bdy),
                typ: loop_type.unwrap_or(Type::Never),
            }
        }
        Expr::Break { bdy, .. } => {
            expect(env.in_loop, BreakOutsideLoop)?;

            let bdy = validate_expr(*bdy, env)?;

            if let Some(loop_type) = env.loop_type {
                expect_type(&bdy, loop_type)?;
            } else {
                *env.loop_type = Some(bdy.typ().clone());
            }

            TExpr::Break {
                bdy: Box::new(bdy),
                typ: Type::Never,
            }
        }
        Expr::Seq { stmt, cnt, .. } => {
            let stmt = validate_expr(*stmt, env)?;
            let cnt = validate_expr(*cnt, env)?;

            TExpr::Seq {
                typ: cnt.typ().clone(),
                stmt: Box::new(stmt),
                cnt: Box::new(cnt),
            }
        }
        Expr::Assign { sym, bnd, .. } => {
            let entry = env.scope.get(&sym).ok_or(UndeclaredVar {
                sym: (*sym).to_string(),
            })?;

            let EnvEntry::Type { typ: _, mutable } = entry else {
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

            let bnd = validate_expr(*bnd, env)?;
            TExpr::Assign {
                sym,
                bnd: Box::new(bnd),
                typ: Type::Unit,
            }
        }
        Expr::Continue { .. } => TExpr::Continue { typ: Type::Never },
        Expr::Return { bdy, .. } => {
            let bdy = validate_expr(*bdy, env)?;
            expect_type(&bdy, env.return_type)?;
            TExpr::Return {
                bdy: Box::new(bdy),
                typ: Type::Never,
            }
        }
        Expr::Struct { sym, fields, .. } => validate_struct(env, sym, fields)?,
        Expr::AccessField { strct, field, .. } => {
            let strct = validate_expr(*strct, env)?;

            let Type::Var { sym } = strct.typ() else {
                return Err(TypeShouldBeStruct {
                    typ: strct.typ().clone().fmap(str::to_string),
                });
            };

            #[rustfmt::skip]
            let EnvEntry::Def { def: TypeDef::Struct { fields: def_fields, .. } } = &env.scope[sym] else {
                return Err(VariableShouldBeStruct { sym: sym.to_string() });
            };

            let Some((_, typ)) = def_fields.iter().find(|&(sym, _)| *sym == field) else {
                return Err(UnknownStructField {
                    sym: sym.to_string(),
                });
            };

            TExpr::AccessField {
                strct: Box::new(strct),
                field,
                typ: typ.clone(),
            }
        }
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    })
}
