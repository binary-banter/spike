mod check_sized;
pub mod error;
mod generate_constraints;
mod resolve_types;
#[cfg(test)]
mod tests;
mod uncover_globals;
pub mod uniquify;
pub mod validate;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr, Lit, Meta, Op, Span};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use derive_more::Display;
use std::collections::HashMap;
use std::str::FromStr;
use crate::passes::validate::generate_constraints::PartialType;

pub struct PrgValidated<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<UniqueSym<'p>, &'p str, ExprValidated<'p>>>,
    pub entry: UniqueSym<'p>,
}

pub struct PrgConstrained<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefConstrained<'p>>,
    pub entry: UniqueSym<'p>,
    pub uf: UnionFind<PartialType<'p>>,
}

pub type DefValidated<'p> = Def<UniqueSym<'p>, &'p str, Meta<Type<UniqueSym<'p>>, ExprValidated<'p>>>;
pub type ExprValidated<'p> = Expr<UniqueSym<'p>, &'p str, TLit, Type<UniqueSym<'p>>>;

pub type DefConstrained<'p> =
    Def<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>, Meta<CMeta, ExprConstrained<'p>>>;
pub type ExprConstrained<'p> = Expr<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>, Lit<'p>, CMeta>;

pub type DefUniquified<'p> =
Def<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>, Meta<Span, ExprUniquified<'p>>>;
pub type ExprUniquified<'p> = Expr<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>,Lit<'p>, Span>;


pub struct CMeta {
    pub span: Span,
    pub index: UnionIndex,
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum TLit {
    #[display(fmt = "{val}")]
    Int { val: i32 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
}

impl TLit {
    /// Returns the integer value if `TLit` is `Int`.
    /// # Panics
    /// Panics if `TLit` is not `Int`.
    #[must_use]
    pub fn int(self) -> i64 {
        if let TLit::Int { val } = self {
            val as i64
        } else {
            panic!()
        }
    }

    /// Returns the boolean value if `TLit` is `Bool`.
    /// # Panics
    /// Panics if `TLit` is not `Bool`.
    #[must_use]
    pub fn bool(self) -> bool {
        if let TLit::Bool { val } = self {
            val
        } else {
            panic!()
        }
    }
}

impl From<TLit> for i64 {
    fn from(value: TLit) -> Self {
        match value {
            TLit::Int { val } => val as i64,
            TLit::Bool { val } => val as i64,
            TLit::Unit => 0,
        }
    }
}

// This implementation is used by the parser.
impl FromStr for TLit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "false" => TLit::Bool { val: false },
            "true" => TLit::Bool { val: true },
            "unit" => TLit::Unit,
            s => TLit::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}
