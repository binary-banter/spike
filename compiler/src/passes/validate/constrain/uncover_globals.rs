use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Spanned, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::uniquify::{PrgUniquified, BUILT_INS};
use crate::passes::validate::DefUniquified;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};

use std::collections::HashMap;

pub struct Env<'a, 'p> {
    pub uf: &'a mut UnionFind<PartialType<'p>>,
    pub scope: &'a mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    pub loop_type: Option<UnionIndex>,
    pub return_type: &'a Spanned<UnionIndex>,
}

pub enum EnvEntry<'p> {
    Type {
        mutable: bool,
        typ: UnionIndex,
    },
    Def {
        def: TypeDef<Spanned<UniqueSym<'p>>, Spanned<&'p str>>,
    },
}

/// Returns a `PushMap` with all the definitions (functions, structs, etc.) in scope.
pub fn uncover_globals<'p>(
    program: &PrgUniquified<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<HashMap<UniqueSym<'p>, EnvEntry<'p>>, TypeError> {
    let builtins = program
        .std
        .iter()
        .map(|(k, v)| {
            let typ = uf.type_to_index(BUILT_INS[k].clone());
            (
                *v,
                EnvEntry::Type {
                    mutable: false,
                    typ,
                },
            )
        })
        .collect::<Vec<_>>();

    // Check for duplicate global names. Names from the standard library are inserted with `None` for its span.
    // todo: Later on this should be replaced by a module system, so that we can span the `use` statement.
    let mut seen = builtins
        .iter()
        .map(|(sym, _)| (sym.sym, None))
        .collect::<HashMap<_, _>>();

    for def in &program.defs {
        let sym = def.sym();

        if let Some(prev_span) = seen.insert(sym.inner.sym, Some(sym.meta)) {
            let error = match prev_span {
                Some(prev_span) => TypeError::DuplicateGlobal {
                    span1: prev_span,
                    span2: sym.meta,
                    sym: sym.inner.sym.to_string(),
                },
                None => TypeError::DuplicateGlobalBuiltin {
                    span: sym.meta,
                    sym: sym.inner.sym.to_string(),
                },
            };

            return Err(error);
        }
    }

    Ok(program
        .defs
        .iter()
        .map(|def| (def.sym().inner, uncover_def(def, uf)))
        .chain(builtins)
        .collect())
}

fn uncover_def<'p>(def: &DefUniquified<'p>, uf: &mut UnionFind<PartialType<'p>>) -> EnvEntry<'p> {
    match def {
        Def::Fn {
            params: args, typ, ..
        } => EnvEntry::Type {
            mutable: false,
            typ: uf.type_to_index(Type::Fn {
                typ: Box::new(typ.clone()),
                params: args.iter().map(|param| param.typ.clone()).collect(),
            }),
        },
        Def::TypeDef { def, .. } => EnvEntry::Def {
            def: (*def).clone(),
        },
    }
}
