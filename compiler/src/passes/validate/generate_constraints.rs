use crate::passes::parse::{Lit, Meta, Span};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uncover_globals::{Env, EnvEntry, uncover_globals};
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{CMeta, DefConstrained, DefUniquified, ExprConstrained, ExprUniquified, PrgConstrained};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use std::collections::HashMap;

pub struct GraphThingy {}

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
    // TODO Any,
}

fn combine_partial_types<'p>(a: PartialType<'p>, b: PartialType<'p>) -> PartialType<'p> {
    todo!()
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
            let mut env = Env {
                uf,
                scope,
                loop_type: &mut None,
                in_loop: false,
                return_type: &typ,
            };

            DefConstrained::Fn {
                sym,
                params,
                bdy: constrain_expr(bdy, &mut env)?,
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
        ExprUniquified::Prim { op, args } => todo!(),
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
