use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Expr, Lit, Meta, Span, UnaryOp};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::MismatchedAssignBinding;
use crate::passes::validate::uncover_globals::{uncover_globals, Env, EnvEntry};
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{
    CMeta, DefConstrained, DefUniquified, ExprConstrained, ExprUniquified, PrgConstrained,
};
use crate::utils::expect::expect;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum PartialType<'p> {
    I64,
    U64,
    Int,
    Bool,
    Unit,
    Never,
    Var {
        sym: UniqueSym<'p>,
    },
    Fn {
        params: Vec<UnionIndex>,
        typ: UnionIndex,
    },
}

impl<'p> PartialType<'p> {
    pub fn to_string(&self, uf: &mut UnionFind<PartialType>) -> String {
        match self {
            PartialType::I64 => "I64".to_string(),
            PartialType::U64 => "U64".to_string(),
            PartialType::Int => "{int}".to_string(),
            PartialType::Bool => "Bool".to_string(),
            PartialType::Unit => "Unit".to_string(),
            PartialType::Never => "Never".to_string(),
            PartialType::Var { sym } => sym.sym.to_string(),
            PartialType::Fn { params, typ } => {
                let params_string = params
                    .iter()
                    .map(|index| {
                        let pt = uf.get(*index).clone();
                        pt.to_string(uf)
                    })
                    .format(", ")
                    .to_string();
                let pt = uf.get(*typ).clone();
                let typ_string = pt.to_string(uf);
                format!("fn({params_string}) -> {typ_string}")
            }
        }
    }
}

impl<'p> PrgUniquified<'p> {
    pub fn constrain(self) -> Result<PrgConstrained<'p>, TypeError> {
        let mut uf = UnionFind::new();
        let mut scope = uncover_globals(&self, &mut uf);

        Ok(PrgConstrained {
            defs: self
                .defs
                .into_iter()
                .map(|def| {
                    constrain_def(def, &mut scope, &mut uf).map(|def| (def.sym().inner, def))
                })
                .collect::<Result<_, _>>()?,
            entry: self.entry,
            uf,
        })
    }
}

fn constrain_def<'p>(
    def: DefUniquified<'p>,
    scope: &mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<DefConstrained<'p>, TypeError> {
    let def = match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => {
            // Put function parameters in scope.
            scope.extend(params.iter().map(|p| {
                (
                    p.sym.inner,
                    EnvEntry::Type {
                        mutable: p.mutable,
                        typ: uf.type_to_index(p.typ.clone()),
                    },
                )
            }));

            // Add return type to env and keep it for error handling.
            let return_index = uf.type_to_index(typ.clone());
            let mut env = Env {
                uf,
                scope,
                loop_type: None,
                return_type: &Meta {
                    inner: return_index,
                    meta: sym.meta,
                }, // TODO replace sym.meta with return type index
            };

            // Constrain body of function.
            let bdy = constrain_expr(bdy, &mut env)?;

            // Return error if function body a type differs from its return type.
            uf.expect_equal(return_index, bdy.meta.index, |r, b| {
                TypeError::MismatchedFnReturn {
                    expect: r,
                    got: b,
                    span_expected: sym.meta,
                    span_got: bdy.meta.span,
                }
            })?;

            Def::Fn {
                sym,
                params,
                bdy,
                typ,
            }
        }
        Def::TypeDef { sym, def } => Def::TypeDef { sym, def },
    };

    Ok(def)
}

fn constrain_expr<'p>(
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
                return Err(TypeError::SymbolShouldBeVariable {
                    span,
                });
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
                return Err(TypeError::SymbolShouldBeVariable {
                    span: sym.meta,
                });
            };

            expect(mutable, TypeError::ModifyImmutable {
                span: sym.meta,
            })?;

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
        Expr::Struct { .. } => todo!(),
        Expr::Variant { .. } => todo!(),
        Expr::AccessField { .. } => todo!(),
        Expr::Switch { .. } => todo!(),
    };

    Ok(meta)
}

fn combine_partial_types<'p>(
    a: PartialType<'p>,
    b: PartialType<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<PartialType<'p>, ()> {
    let typ = match (a, b) {
        (PartialType::I64, PartialType::I64 | PartialType::Int) => PartialType::I64,
        (PartialType::Int, PartialType::I64) => PartialType::I64,
        (PartialType::U64, PartialType::U64 | PartialType::Int) => PartialType::U64,
        (PartialType::Int, PartialType::U64) => PartialType::U64,
        (PartialType::Int, PartialType::Int) => PartialType::Int,
        (PartialType::Bool, PartialType::Bool) => PartialType::Bool,
        (PartialType::Unit, PartialType::Unit) => PartialType::Unit,
        (PartialType::Never, t) => t.clone(),
        (t, PartialType::Never) => t.clone(),
        (PartialType::Var { sym: sym_a }, PartialType::Var { sym: sym_b }) if sym_a == sym_b => {
            PartialType::Var { sym: sym_a }
        }
        (
            PartialType::Fn {
                params: params_a,
                typ: typ_a,
            },
            PartialType::Fn {
                params: params_b,
                typ: typ_b,
            },
        ) => {
            if params_a.len() != params_b.len() {
                return Err(());
            }

            let params = params_a
                .into_iter()
                .zip(params_b)
                .map(|(param_a, param_b)| uf.try_union_by(param_a, param_b, combine_partial_types))
                .collect::<Result<_, _>>()?;

            let typ = uf.try_union_by(typ_a, typ_b, combine_partial_types)?;

            PartialType::Fn { params, typ }
        }
        _ => return Err(()),
    };

    Ok(typ)
}

impl<'p> UnionFind<PartialType<'p>> {
    pub fn expect_equal(
        &mut self,
        a: UnionIndex,
        b: UnionIndex,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        self.try_union_by(a, b, combine_partial_types).map_err(|_| {
            let typ_a = self.get(a).clone();
            let str_a = typ_a.to_string(self);
            let typ_b = self.get(b).clone();
            let str_b = typ_b.to_string(self);
            map_err(str_a, str_b)
        })
    }

    pub fn expect_type(
        &mut self,
        a: UnionIndex,
        t: Type<Meta<Span, UniqueSym<'p>>>,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        let t_index = self.type_to_index(t);
        self.expect_equal(a, t_index, map_err)
    }

    pub fn expect_partial_type(
        &mut self,
        a: UnionIndex,
        t: PartialType<'p>,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        let t_index = self.add(t);
        self.expect_equal(a, t_index, map_err)
    }

    pub fn type_to_index(&mut self, t: Type<Meta<Span, UniqueSym<'p>>>) -> UnionIndex {
        let pt = match t {
            Type::I64 => PartialType::I64,
            Type::U64 => PartialType::U64,
            Type::Bool => PartialType::Bool,
            Type::Unit => PartialType::Unit,
            Type::Never => PartialType::Never,
            Type::Fn { params, typ } => PartialType::Fn {
                params: params
                    .into_iter()
                    .map(|param| self.type_to_index(param))
                    .collect(),
                typ: self.type_to_index(*typ),
            },
            Type::Var { sym } => PartialType::Var { sym: sym.inner },
        };

        self.add(pt)
    }
}
