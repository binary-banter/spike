pub mod error;
// mod uncover_globals;
// mod validate_access_field;
// mod validate_apply;
// mod validate_assign;
// mod validate_break;
// mod validate_continue;
// mod validate_expr;
// mod validate_if;
// mod validate_let;
// mod validate_lit;
// mod validate_loop;
// mod validate_prim;
// mod validate_return;
// mod validate_seq;
// mod validate_struct;
// mod validate_switch;
// mod validate_type;
// mod validate_typedef;
// mod validate_var;
// mod validate_variant;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, PrgParsed, Spanned, TypeDef};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::uncover_globals::uncover_globals;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::validate_typedef::validate_typedef;
use crate::passes::validate::{PrgValidated, TExpr};
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
    Type { mutable: bool, typ: Type<&'p str> },
    Def { def: TypeDef<&'p str> },
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
    pub(super) fn type_check(self) -> Result<PrgValidated<'p>, TypeError> {
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
                                bdy: bdy.inner,
                                typ,
                            },
                        ))
                    },
                ),
                Def::TypeDef { sym, def } => Ok((
                    sym,
                    Def::TypeDef {
                        sym,
                        def: validate_typedef(sym, def, &scope)?,
                    },
                )),
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(PrgValidated {
            defs,
            entry: self.entry,
        })
    }
}

pub fn expect_type_eq<'p>(
    e1: &Spanned<TExpr<'p, &'p str>>,
    e2: &Spanned<TExpr<'p, &'p str>>,
) -> Result<Type<&'p str>, TypeError> {
    let t1 = e1.inner.typ();
    let t2 = e2.inner.typ();

    expect(
        t1 == t2,
        MismatchedTypes {
            t1: t1.clone().fmap(str::to_string),
            t2: t2.clone().fmap(str::to_string),
            span_t1: s(e1.span),
            span_t2: s(e2.span),
        },
    )?;

    Ok(t1.clone())
}

pub fn expect_type<'p>(
    expr: &Spanned<TExpr<'p, &'p str>>,
    expected: &Type<&'p str>,
) -> Result<(), TypeError> {
    let typ = expr.inner.typ();
    expect(
        typ == expected,
        MismatchedType {
            got: typ.clone().fmap(str::to_string),
            expect: expected.clone().fmap(str::to_string),
            span: s(expr.span),
        },
    )
}

pub fn s(span: (usize, usize)) -> (usize, usize) {
    (span.0, span.1 - span.0)
}
