use std::collections::{HashMap, HashSet};
use crate::passes::parse::{BinaryOp, Expr, Lit, Meta, Span, TypeDef, UnaryOp};
use crate::passes::parse::types::Type;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::MismatchedAssignBinding;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::uncover_globals::{Env, EnvEntry};
use crate::utils::expect::expect;

pub fn constrain_expr<'p>(
    expr: Meta<Span, ExprUniquified<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let span = expr.meta;

    let meta = match expr.inner {
        Expr::Lit { val } => {
            let typ = match &val {
                Lit::Int { typ, .. } => typ.clone().unwrap_or(PartialType::Int),
                Lit::Bool { .. } => PartialType::Bool,
                Lit::Unit => PartialType::Unit,
            };
            let index = env.uf.add(typ);
            Meta {
                meta: CMeta { span, index },
                inner: ExprConstrained::Lit { val },
            }
        }
        Expr::Var { sym } => {
            let EnvEntry::Type { typ, .. } = env.scope[&sym.inner] else {
                return Err(TypeError::SymbolShouldBeVariable { span });
            };
            Meta {
                meta: CMeta { span, index: typ },
                inner: ExprConstrained::Var { sym },
            }
        }
        Expr::UnaryOp { op, expr } => {
            let typ = match op {
                UnaryOp::Neg => Type::I64,
                UnaryOp::Not => Type::Bool,
            };
            let expr = constrain_expr(*expr, env)?;

            env.uf.expect_type(expr.meta.index, typ, |got, expect| {
                TypeError::OperandExpect {
                    expect,
                    got,
                    op: op.to_string(),
                    span_op: span,
                    span_arg: expr.meta.span,
                }
            })?;

            Meta {
                meta: CMeta {
                    span,
                    index: expr.meta.index,
                },
                inner: ExprConstrained::UnaryOp {
                    op,
                    expr: Box::new(expr),
                },
            }
        }
        Expr::BinaryOp {
            op,
            exprs: [lhs, rhs],
        } => {
            // input: None = Any but equal, Some = expect this
            // output: None = Same as input, Some = this
            let (input, output) = match op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    (Some(PartialType::Int), None)
                }
                BinaryOp::LAnd | BinaryOp::LOr | BinaryOp::Xor => (Some(PartialType::Bool), None),
                BinaryOp::GT | BinaryOp::GE | BinaryOp::LE | BinaryOp::LT => {
                    (Some(PartialType::Int), Some(PartialType::Bool))
                }
                BinaryOp::EQ | BinaryOp::NE => (None, Some(PartialType::Bool)),
            };

            let e1 = constrain_expr(*lhs, env)?;
            let e2 = constrain_expr(*rhs, env)?;

            // Check inputs satisfy constraints
            if let Some(input) = input {
                let mut check = |expr: &Meta<CMeta, ExprConstrained<'p>>| {
                    env.uf
                        .expect_partial_type(expr.meta.index, input.clone(), |got, expect| {
                            TypeError::OperandExpect {
                                expect,
                                got,
                                op: op.to_string(),
                                span_op: span,
                                span_arg: expr.meta.span,
                            }
                        })
                };

                check(&e1)?;
                check(&e2)?;
            }

            // Check inputs equal
            let input_index = env
                .uf
                .expect_equal(e1.meta.index, e2.meta.index, |lhs, rhs| {
                    TypeError::OperandEqual {
                        lhs,
                        rhs,
                        op: op.to_string(),
                        span_op: span,
                        span_lhs: e1.meta.span,
                        span_rhs: e2.meta.span,
                    }
                })?;

            // Generate output index
            let output_index = match output {
                None => input_index,
                Some(e) => env.uf.add(e),
            };

            Meta {
                meta: CMeta {
                    span,
                    index: output_index,
                },
                inner: ExprConstrained::BinaryOp {
                    op,
                    exprs: [e1, e2].map(Box::new),
                },
            }
        }
        Expr::Let {
            sym,
            mutable,
            typ,
            bnd,
            bdy,
        } => {
            let bnd = constrain_expr(*bnd, env)?;

            if let Some(typ) = &typ {
                env.uf.expect_type(bnd.meta.index, typ.clone(), |got, _| {
                    TypeError::MismatchedLetBinding {
                        got,
                        span_expected: (0, 0), //TODO span of typ
                        span_got: bnd.meta.span,
                    }
                })?;
            }

            env.scope.insert(
                sym.inner,
                EnvEntry::Type {
                    mutable,
                    typ: bnd.meta.index,
                },
            );
            let bdy = constrain_expr(*bdy, env)?;

            Meta {
                meta: CMeta {
                    span,
                    index: bdy.meta.index,
                },
                inner: ExprConstrained::Let {
                    sym,
                    mutable,
                    typ,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                },
            }
        }
        Expr::If { cnd, thn, els } => {
            let cnd = constrain_expr(*cnd, env)?;

            env.uf.expect_type(cnd.meta.index, Type::Bool, |got, _| {
                TypeError::IfExpectBool {
                    got,
                    span_got: cnd.meta.span,
                }
            })?;

            let thn = constrain_expr(*thn, env)?;
            let els = constrain_expr(*els, env)?;

            let out_index =
                env.uf
                    .expect_equal(thn.meta.index, els.meta.index, |thn_type, els_type| {
                        TypeError::IfExpectEqual {
                            thn: thn_type,
                            els: els_type,
                            span_thn: thn.meta.span,
                            span_els: els.meta.span,
                        }
                    })?;

            Meta {
                meta: CMeta {
                    span,
                    index: out_index,
                },
                inner: ExprConstrained::If {
                    cnd: Box::new(cnd),
                    thn: Box::new(thn),
                    els: Box::new(els),
                },
            }
        }
        Expr::Apply { fun, args } => {
            let fun = constrain_expr(*fun, env)?;
            let args: Vec<_> = args
                .into_iter()
                .map(|arg| constrain_expr(arg, env))
                .collect::<Result<_, _>>()?;

            let p_typ = env.uf.get(fun.meta.index).clone();
            let PartialType::Fn { params, typ } = p_typ else {
                return Err(TypeError::TypeMismatchExpectFn {
                    got: p_typ.to_string(&mut env.uf),
                    span_got: fun.meta.span,
                });
            };

            expect(
                params.len() == args.len(),
                TypeError::ArgCountMismatch {
                    got: args.len(),
                    expected: params.len(),
                    span, // todo: maybe highlight only the args and params?
                },
            )?;

            for (arg, param_type) in args.iter().zip(params.iter()) {
                env.uf
                    .expect_equal(arg.meta.index, *param_type, |arg_type, param_type| {
                        TypeError::FnArgExpect {
                            arg: arg_type,
                            param: param_type,
                            span_arg: arg.meta.span,
                        }
                    })?;
            }

            Meta {
                meta: CMeta { span, index: typ },
                inner: ExprConstrained::Apply {
                    fun: Box::new(fun),
                    args,
                },
            }
        }
        Expr::Loop { bdy } => {
            let loop_type = env.uf.add(PartialType::Never);

            let mut env = Env {
                uf: env.uf,
                scope: env.scope,
                loop_type: Some(loop_type),
                return_type: env.return_type,
            };

            let bdy = constrain_expr(*bdy, &mut env)?;

            Meta {
                meta: CMeta {
                    span,
                    index: loop_type,
                },
                inner: ExprConstrained::Loop { bdy: Box::new(bdy) },
            }
        }
        Expr::Break { bdy } => {
            let Some(loop_type) = env.loop_type else {
                return Err(TypeError::BreakOutsideLoop { span });
            };

            let bdy = constrain_expr(*bdy, env)?;
            env.uf
                .expect_equal(bdy.meta.index, loop_type, |got, expect| {
                    TypeError::TypeMismatchLoop {
                        expect,
                        got,
                        span_break: bdy.meta.span,
                    }
                })?;

            Meta {
                meta: CMeta {
                    span,
                    index: env.uf.add(PartialType::Never),
                },
                inner: ExprConstrained::Break { bdy: Box::new(bdy) },
            }
        }
        Expr::Continue => {
            expect(
                env.loop_type.is_some(),
                TypeError::ContinueOutsideLoop { span },
            )?;

            Meta {
                meta: CMeta {
                    span,
                    index: env.uf.add(PartialType::Never),
                },
                inner: ExprConstrained::Continue,
            }
        }
        Expr::Return { bdy } => {
            let bdy = constrain_expr(*bdy, env)?;

            env.uf
                .expect_equal(bdy.meta.index, env.return_type.inner, |bdy_typ, rtrn| {
                    TypeError::MismatchedFnReturn {
                        got: bdy_typ,
                        expect: rtrn,
                        span_got: bdy.meta.span,
                        span_expected: env.return_type.meta, //TODO span of return type, should be passed via env
                    }
                })?;

            Meta {
                meta: CMeta {
                    span,
                    index: env.uf.add(PartialType::Never),
                },
                inner: ExprConstrained::Return { bdy: Box::new(bdy) },
            }
        }
        Expr::Seq { stmt, cnt } => {
            let stmt = constrain_expr(*stmt, env)?;
            let cnt = constrain_expr(*cnt, env)?;

            Meta {
                meta: CMeta {
                    span,
                    index: cnt.meta.index,
                },
                inner: ExprConstrained::Seq {
                    stmt: Box::new(stmt),
                    cnt: Box::new(cnt),
                },
            }
        }
        Expr::Assign { sym, bnd } => {
            let bnd = constrain_expr(*bnd, env)?;

            let EnvEntry::Type { mutable, typ } = env.scope[&sym.inner] else {
                return Err(TypeError::SymbolShouldBeVariable { span: sym.meta });
            };

            expect(mutable, TypeError::ModifyImmutable { span: sym.meta })?;

            env.uf
                .expect_equal(typ, bnd.meta.index, |sym_typ, bnd_type| {
                    MismatchedAssignBinding {
                        expect: sym_typ,
                        got: bnd_type,
                        span_expected: sym.meta,
                        span_got: bnd.meta.span,
                    }
                })?;

            let typ = env.uf.add(PartialType::Unit);

            Meta {
                meta: CMeta { span, index: typ },
                inner: ExprConstrained::Assign {
                    sym,
                    bnd: Box::new(bnd),
                },
            }
        }
        Expr::Struct { sym, fields } => {
            // Get the `EnvEntry` from the scope.
            // This should exist after uniquify, but could potentially not be a struct definition.
            let EnvEntry::Def {
                def: TypeDef::Struct { fields: def_fields },
            } = &env.scope[&sym.inner]
            else {
                return Err(TypeError::SymbolShouldBeStruct { span });
            };

            let def_fields = def_fields
                .iter()
                .map(|(field_sym, field_typ)| {
                    (field_sym.inner, (field_sym.meta, field_typ.clone()))
                })
                .collect::<HashMap<_, _>>();

            // Set to keep track of fields in the struct constructor. Used to make sure no duplicates occur.
            let mut seen_fields = HashSet::new();

            let fields = fields
                .into_iter()
                .map(|(field_sym, field_bnd)| {
                    let field_bnd = constrain_expr(field_bnd, env)?;

                    expect(
                        seen_fields.insert(field_sym.inner),
                        TypeError::VariableConstructDuplicateField {
                            sym: field_sym.to_string(),
                            span: field_sym.meta,
                        },
                    )?;

                    let Some((def_span, def_typ)) = def_fields.get(field_sym.inner) else {
                        return Err(TypeError::UnknownStructField {
                            sym: field_sym.to_string(),
                            span: field_sym.meta,
                        });
                    };

                    env.uf.expect_type(
                        field_bnd.meta.index,
                        def_typ.clone(),
                        |field_typ, def_typ| TypeError::MismatchedStructField {
                            expect: def_typ,
                            got: field_typ,
                            span_expected: *def_span,
                            span_got: field_sym.meta,
                        },
                    )?;

                    Ok((field_sym, field_bnd))
                })
                .collect::<Result<Vec<_>, _>>()?;

            // Verify that all fields from the struct definition are present.
            for (def_sym, (def_span, _)) in def_fields {
                expect(
                    seen_fields.contains(def_sym),
                    TypeError::VariableConstructMissingField {
                        sym: def_sym.to_string(),
                        struct_span: sym.meta,
                        def_span,
                    },
                )?;
            }

            let index = env.uf.add(PartialType::Var { sym: sym.inner });

            Meta {
                meta: CMeta { span, index },
                inner: ExprConstrained::Struct { sym, fields },
            }
        }
        Expr::AccessField { strct, field } => {
            let strct = constrain_expr(*strct, env)?;

            let PartialType::Var { sym } = env.uf.get(strct.meta.index) else {
                return Err(TypeError::SymbolShouldBeStruct {
                    span: strct.meta.span,
                });
            };

            let EnvEntry::Def {
                def: TypeDef::Struct {
                    fields: def_fields, ..
                },
            } = &env.scope[sym]
            else {
                return Err(TypeError::SymbolShouldBeStruct {
                    span: strct.meta.span,
                });
            };

            let Some((_, typ)) = def_fields.iter().find(|(sym, _)| sym.inner == field.inner) else {
                return Err(TypeError::UnknownStructField {
                    sym: field.inner.to_string(),
                    span: field.meta,
                });
            };

            let index = env.uf.type_to_index(typ.clone());
            Meta {
                meta: CMeta { span, index },
                inner: ExprConstrained::AccessField { strct: Box::new(strct), field },
            }
        },
        Expr::Variant { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    };

    Ok(meta)
}
