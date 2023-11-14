use crate::passes::parse::{Lit, Meta, Span};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::MismatchedFnReturn;
use crate::passes::validate::uncover_globals::{uncover_globals, Env, EnvEntry};
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{
    type_to_index, CMeta, DefConstrained, DefUniquified, ExprConstrained, ExprUniquified,
    PrgConstrained,
};
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
            PartialType::Var { sym } => format!("{}", sym.sym),
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
        DefUniquified::Fn {
            sym,
            params,
            typ,
            bdy,
        } => {
            scope.extend(params.iter().map(|p| {
                (
                    p.sym.inner,
                    EnvEntry::Type {
                        mutable: p.mutable,
                        typ: type_to_index(p.typ.clone(), uf),
                    },
                )
            }));

            let return_index = type_to_index(typ.clone(), uf);
            let mut env = Env {
                uf,
                scope,
                return_type: return_index,
            };

            let bdy = constrain_expr(bdy, &mut env)?;

            uf.try_union_by(return_index, bdy.meta.index, combine_partial_types)
                .map_err(|_| MismatchedFnReturn {
                    expect: format!("{typ}"),
                    got: format!("{}", "bananas"),
                    span_expected: (0, 0),
                    span_got: (0, 0),
                })?;

            DefConstrained::Fn {
                sym,
                params,
                bdy,
                typ,
            }
        }
        DefUniquified::TypeDef { sym, def } => DefConstrained::TypeDef { sym, def },
    };

    Ok(def)
}

fn constrain_expr<'p>(
    expr: Meta<Span, ExprUniquified<'p>>,
    env: &mut Env<'_, 'p>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    Ok(match expr.inner {
        ExprUniquified::Lit { val } => {
            let typ = match val {
                Lit::Int { .. } => PartialType::Int,
                Lit::Bool { .. } => PartialType::Bool,
                Lit::Unit => PartialType::Unit,
            };
            let index = env.uf.add(typ);
            Meta {
                meta: CMeta {
                    span: expr.meta,
                    index,
                },
                inner: ExprConstrained::Lit { val },
            }
        }
        ExprUniquified::Var { sym } => {
            let EnvEntry::Type { typ, .. } = env.scope[&sym.inner] else {
                panic!();
            };
            Meta {
                meta: CMeta {
                    span: expr.meta,
                    index: typ,
                },
                inner: ExprConstrained::Var { sym },
            }
        }
        ExprUniquified::UnaryOp { op, expr } => todo!(),
        ExprUniquified::BinaryOp { op, exprs } => todo!(),
        // ExprUniquified::Prim { op, args } if args.len() == 2 => {
        //     let (pt, lhs, rhs) = match op {
        //         Op::Plus => (PartialType::Int, PartialType::Int, PartialType::Int),
        //         Op::Minus => PartialType::Int,
        //         Op::Mul => PartialType::Int,
        //         Op::Div => PartialType::Int,
        //         Op::Mod => PartialType::Int,
        //         Op::LAnd => todo!(),
        //         Op::LOr => todo!(),
        //         Op::Xor => todo!(),
        //         Op::GT => PartialType::Int,
        //         Op::GE => PartialType::Int,
        //         Op::EQ => todo!(),
        //         Op::LE => PartialType::Int,
        //         Op::LT => PartialType::Int,
        //         Op::NE => todo!(),
        //         Op::Read | Op::Print | Op::Not => unreachable!(),
        //     };
        //
        //     let index = env.uf.add(pt);
        //
        //     Meta {
        //         meta: CMeta{ span: expr.meta, index },
        //         inner: ExprConstrained::Prim { op, args: args.into_iter().map(|arg| match arg {
        //
        //         })},
        //     }
        // },
        ExprUniquified::Let { sym, bnd, bdy, .. } => todo!(),
        ExprUniquified::If { .. } => todo!(),
        ExprUniquified::Apply { .. } => todo!(),
        ExprUniquified::Loop { .. } => todo!(),
        ExprUniquified::Break { .. } => todo!(),
        ExprUniquified::Continue => todo!(),
        ExprUniquified::Return { .. } => todo!(),
        ExprUniquified::Seq { .. } => todo!(),
        ExprUniquified::Assign { .. } => todo!(),
        ExprUniquified::Struct { .. } => todo!(),
        ExprUniquified::Variant { .. } => todo!(),
        ExprUniquified::AccessField { .. } => todo!(),
        ExprUniquified::Switch { .. } => todo!(),
    })
}

// uf: &mut UnionFind<PartialType<'p>>
fn combine_partial_types<'p>(
    a: PartialType<'p>,
    b: PartialType<'p>,
    uf: &mut UnionFind<PartialType<'p>>
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
                return Err(())
            }

            let params = params_a.into_iter().zip(params_b).map(|(param_a, param_b)| {
                uf.try_union_by(param_a, param_b, combine_partial_types)
            }).collect::<Result<_,_>>()?;

            let typ = uf.try_union_by(typ_a, typ_b, combine_partial_types)?;

            PartialType::Fn {
                params,
                typ,
            }
        }
        _ => return Err(()),
    };

    Ok(typ)
}
