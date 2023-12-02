use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, PrgParsed, Spanned};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::{NoMain, UndeclaredVar};
use crate::passes::validate::DefUniquified;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::utils::push_map::PushMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;

mod def;
mod expr;
mod r#fn;
mod r#type;
mod typedef;

pub struct PrgUniquified<'p> {
    /// The global program definitions.
    pub defs: Vec<DefUniquified<'p>>,
    /// The symbol representing the entry point of the program.
    pub entry: UniqueSym<'p>,
}

impl<'p> PrgParsed<'p> {
    pub fn uniquify(self) -> Result<PrgUniquified<'p>, TypeError> {
        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|def| (def.sym().inner, gen_sym(def.sym().inner)))
        );

        let entry = *scope.get(&"main").ok_or(NoMain)?;

        Ok(PrgUniquified {
            defs: self
                .defs
                .into_iter()
                .map(|def| def::uniquify_def(def, &mut scope))
                .collect::<Result<_, _>>()?,
            entry,
        })
    }
}

fn try_get<'p>(
    sym: Spanned<&'p str>,
    scope: &PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Spanned<UniqueSym<'p>>, TypeError> {
    scope
        .get(&sym.inner)
        .ok_or(UndeclaredVar {
            sym: sym.inner.to_string(),
            span: sym.meta,
        })
        .map(|&inner| Meta {
            meta: sym.meta,
            inner,
        })
}

fn gen_spanned_sym(sym: Spanned<&str>) -> Spanned<UniqueSym> {
    Meta {
        inner: gen_sym(sym.inner),
        meta: sym.meta,
    }
}
