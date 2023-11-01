use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr, PrgParsed};
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::{uncover_globals, util, validate_type, PrgTypeChecked};
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;

pub struct Env<'a, 'p> {
    pub scope: &'a mut PushMap<&'p str, EnvEntry<'p>>,
    pub loop_type: &'a mut Option<Type<&'p str>>,
    pub in_loop: bool,
    pub return_type: &'a Type<&'p str>,
}

pub enum EnvEntry<'p> {
    Type {
        mutable: bool,
        typ: Type<&'p str>,
    },
    Def {
        def: &'p Def<&'p str, Expr<&'p str>>,
    },
}

impl<'a, 'p> Env<'a, 'p> {
    pub fn push<O>(
        &mut self,
        k: &'p str,
        v: EnvEntry<'p>,
        sub: impl FnOnce(&mut Env<'_, 'p>) -> O,
    ) -> O {
        self.scope.push(k, v, |scope| {
            sub(&mut Env {
                scope,
                loop_type: self.loop_type,
                in_loop: self.in_loop,
                return_type: self.return_type,
            })
        })
    }
}

impl<'p> PrgParsed<'p> {
    pub fn type_check(self) -> Result<PrgTypeChecked<'p>, TypeError> {
        let mut scope = uncover_globals::uncover_globals(&self)?;

        self.defs
            .iter()
            .map(|def| match def {
                Def::Fn {
                    ref params,
                    ref bdy,
                    ref typ,
                    ..
                } => scope.push_iter(
                    params.iter().map(|p| {
                        (
                            p.sym,
                            EnvEntry::Type {
                                mutable: p.mutable,
                                typ: p.typ.clone(),
                            },
                        )
                    }),
                    |scope| {
                        let mut env = Env {
                            scope,
                            loop_type: &mut None,
                            in_loop: false,
                            return_type: typ,
                        };

                        util::expect_type(bdy, typ.clone(), &mut env)
                    },
                ),
                Def::Struct { fields: types, .. }
                | Def::Enum {
                    variants: types, ..
                } => {
                    for (_, typ) in types {
                        validate_type::validate_type(typ, &scope)?;
                    }
                    Ok(())
                }
            })
            .collect::<Result<(), _>>()?;

        let defs = self
            .defs
            .into_iter()
            .map(|def| (*def.sym(), def))
            .collect::<HashMap<_, _>>();

        expect(defs.contains_key("main"), NoMain)?;

        Ok(PrgTypeChecked {
            defs,
            entry: self.entry,
        })
    }
}
