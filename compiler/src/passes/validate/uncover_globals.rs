use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Meta, Span, TypeDef};
use crate::passes::validate::generate_constraints::PartialType;
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{type_to_index, DefUniquified};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use std::collections::HashMap;

pub struct Env<'a, 'p> {
    pub uf: &'a mut UnionFind<PartialType<'p>>,
    pub scope: &'a mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    // pub loop_type: &'a mut Option<Meta<Span, UniqueSym<'p>>>,
    // pub in_loop: bool,
    pub return_type: UnionIndex,
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
