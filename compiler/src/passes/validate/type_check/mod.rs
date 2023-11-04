pub mod error;
mod uncover_globals;
mod validate_expr;
mod validate_prim;
mod validate_struct;
mod validate_type;
mod validate_typedef;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, PrgParsed, TypeDef};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::uncover_globals::uncover_globals;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::{PrgTypeChecked, TExpr};
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;
use crate::passes::validate::type_check::validate_typedef::validate_typedef;

pub struct Env<'a, 'p> {
    pub scope: &'a mut PushMap<&'p str, EnvEntry<'p>>,
    pub loop_type: &'a mut Option<Type<&'p str>>,
    pub in_loop: bool,
    pub return_type: &'a Type<&'p str>,
}

pub enum EnvEntry<'p> {
    Type { mutable: bool, typ: Type<&'p str> },
    Def { def: TypeDef<'p, &'p str> },
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

        let defs = self
            .defs
            .into_iter()
            .map(|def| match def {
                Def::Fn {
                    sym,
                    params,
                    bdy,
                    typ,
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
                        Ok((
                            sym,
                            Def::Fn {
                                sym,
                                params,
                                bdy,
                                typ,
                            },
                        ))
                    },
                ),
                Def::TypeDef { sym, def } => Ok((
                    sym,
                    Def::TypeDef {
                        sym,
                        def: validate_typedef(sym, def, &mut scope)?,
                    })),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(PrgTypeChecked {
            defs,
            entry: self.entry,
        })
    }
}

pub fn expect_type_eq<'p>(
    e1: &TExpr<'p, &'p str>,
    e2: &TExpr<'p, &'p str>,
) -> Result<Type<&'p str>, TypeError> {
    let t1 = e1.typ();
    let t2 = e2.typ();
    expect(
        t1 == t2,
        TypeMismatchEqual {
            t1: t1.clone().fmap(str::to_string),
            t2: t2.clone().fmap(str::to_string),
        },
    )?;
    Ok(t1.clone())
}

pub fn expect_type<'p>(
    expr: &TExpr<'p, &'p str>,
    expected: &Type<&'p str>,
) -> Result<(), TypeError> {
    let t = expr.typ();
    expect(
        t == expected,
        TypeMismatchExpect {
            got: t.clone().fmap(str::to_string),
            expect: expected.clone().fmap(str::to_string),
        },
    )
}
