use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, DefUniquified, Meta, Span, TypeDef};
use std::collections::HashMap;
use crate::passes::validate::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;

pub struct Env<'a, 'p> {
    pub scope: &'a mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    pub loop_type: &'a mut Option<Meta<Span, UniqueSym<'p>>>,
    pub in_loop: bool,
    pub return_type: &'a Type<Meta<Span, UniqueSym<'p>>>,
}

pub enum EnvEntry<'p> {
    Type { mutable: bool, typ: Type<Meta<Span, UniqueSym<'p>>> },
    Def { def: TypeDef<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>> },
}

impl<'a, 'p> Env<'a, 'p> {
    pub fn insert<O>(&mut self, k: UniqueSym<'p>, v: EnvEntry<'p>) {
        self.scope.insert(k, v);
    }
}

/// Returns a `PushMap` with all the functions in scope.
pub fn uncover_globals<'p>(program: &PrgUniquified<'p>) -> HashMap<UniqueSym<'p>, EnvEntry<'p>> {
    program.defs.iter().map(|def| (def.sym().inner, uncover_def(def))).collect()
}

fn uncover_def<'p>(def: &DefUniquified<'p>) -> EnvEntry<'p> {
    match def {
        Def::Fn {
            params: args,
            typ,
            ..
        } => {
            EnvEntry::Type {
                mutable: false,
                typ: Type::Fn {
                    typ: Box::new(typ.clone()),
                    params: args.iter().map(|param| param.typ.clone()).collect(),
                }}
        }
        Def::TypeDef { def, .. } => {
            EnvEntry::Def {
                def: (*def).clone(),
            }
        }
    }
}