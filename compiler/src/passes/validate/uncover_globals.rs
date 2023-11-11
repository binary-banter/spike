use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, DefUniquified, Meta, Span, TypeDef};
use crate::passes::validate::generate_constraints::PartialType;
use crate::passes::validate::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use std::collections::HashMap;

pub struct Env<'a, 'p> {
    pub uf: &'a mut UnionFind<PartialType<'p>>,
    pub scope: &'a mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    pub loop_type: &'a mut Option<Meta<Span, UniqueSym<'p>>>,
    pub in_loop: bool,
    pub return_type: &'a Type<Meta<Span, UniqueSym<'p>>>,
}

pub enum EnvEntry<'p> {
    Type {
        mutable: bool,
        typ: UnionIndex,
    },
    Def {
        def: TypeDef<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>>,
    },
}

impl<'a, 'p> Env<'a, 'p> {
    pub fn insert<O>(&mut self, k: UniqueSym<'p>, v: EnvEntry<'p>) {
        self.scope.insert(k, v);
    }
}

/// Returns a `PushMap` with all the functions in scope.
pub fn uncover_globals<'p>(
    program: &PrgUniquified<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> HashMap<UniqueSym<'p>, EnvEntry<'p>> {
    program
        .defs
        .iter()
        .map(|def| (def.sym().inner, uncover_def(def, uf)))
        .collect()
}

fn uncover_def<'p>(def: &DefUniquified<'p>, uf: &mut UnionFind<PartialType<'p>>) -> EnvEntry<'p> {
    match def {
        Def::Fn {
            params: args, typ, ..
        } => EnvEntry::Type {
            mutable: false,
            typ: type_to_index(
                Type::Fn {
                    typ: Box::new(typ.clone()),
                    params: args.iter().map(|param| param.typ.clone()).collect(),
                },
                uf,
            ),
        },
        Def::TypeDef { def, .. } => EnvEntry::Def {
            def: (*def).clone(),
        },
    }
}

fn type_to_index<'p>(
    t: Type<Meta<Span, UniqueSym<'p>>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> UnionIndex {
    let pt = match t {
        Type::I64 => PartialType::I64,
        Type::U64 => PartialType::U64,
        Type::Bool => PartialType::Bool,
        Type::Unit => PartialType::Unit,
        Type::Never => PartialType::Never,
        Type::Fn { params, typ } => PartialType::Fn {
            params: params
                .into_iter()
                .map(|param| type_to_index(param, uf))
                .collect(),
            typ: type_to_index(*typ, uf),
        },
        Type::Var { sym } => PartialType::Var { sym: sym.inner },
    };

    uf.add(pt)
}
