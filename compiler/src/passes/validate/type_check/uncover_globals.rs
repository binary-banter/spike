use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, PrgParsed};
use crate::passes::validate::type_check::EnvEntry;
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use std::collections::{HashMap, HashSet};

/// Returns a `PushMap` with all the functions in scope.
pub fn uncover_globals<'p>(
    program: &PrgParsed<'p>,
) -> Result<PushMap<&'p str, EnvEntry<'p>>, TypeError> {
    let mut globals = HashMap::new();

    for def in &program.defs {
        match def {
            Def::Fn {
                sym,
                params: args,
                typ,
                ..
            } => {
                let signature = Type::Fn {
                    typ: Box::new(typ.clone()),
                    params: args.iter().map(|param| param.typ.clone()).collect(),
                };
                expect(
                    globals
                        .insert(
                            *sym,
                            EnvEntry::Type {
                                mutable: false,
                                typ: signature,
                            },
                        )
                        .is_none(),
                    DuplicateFunction {
                        sym: (*sym).to_string(),
                    },
                )?;

                let mut arg_syms = HashSet::new();
                expect(
                    args.iter().all(|param| arg_syms.insert(param.sym)),
                    DuplicateArg {
                        sym: (*sym).to_string(),
                    },
                )?;
            }
            Def::TypeDef { sym, def } => {
                globals.insert(
                    sym,
                    EnvEntry::Def {
                        def: (*def).clone(),
                    },
                );
            }
        }
    }

    Ok(PushMap::from(globals))
}
