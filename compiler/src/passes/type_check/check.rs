use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr, PrgParsed, TypeDef};
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::uncover_globals::uncover_globals;
use crate::passes::type_check::util::expect_type;
use crate::passes::type_check::validate_type::validate_type;
use crate::passes::type_check::PrgTypeChecked;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;
use crate::passes::type_check::validate_expr::validate_expr;

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
        def: TypeDef<'p, &'p str>,
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
        let mut scope = uncover_globals(&self)?;

        let defs = self.defs.into_iter().map(|def| match def {
            Def::Fn {
                sym,
                params,
                bdy,
                typ
            } => scope.push_iter(
                params.clone().iter().map(|p| {
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
                        return_type: &typ,
                    };
                    let bdy = validate_expr(bdy, &mut env)?;
                    expect_type(&bdy, &typ)?;
                    Ok((sym, Def::Fn {
                        sym,
                        params,
                        bdy,
                        typ
                    }))
                },
            ),
            Def::TypeDef { sym, def} => Ok((sym, Def::TypeDef { sym, def: match def {
                TypeDef::Struct { fields } => {
                    fields.iter().try_for_each(|(_, typ)| validate_type(typ, &scope))?;
                    TypeDef::Struct { fields}
                },
                TypeDef::Enum { .. } => todo!(),
            }}))
        }).collect::<Result<HashMap<_, _>, _>>()?;

        expect(defs.contains_key("main"), NoMain)?;

        Ok(PrgTypeChecked {
            defs,
            entry: self.entry,
        })
    }
}
