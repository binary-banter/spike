mod check_sized;
mod constrain;
pub mod error;
pub mod partial_type;
mod resolve;
#[cfg(test)]
mod tests;
mod uniquify;
pub mod validate;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Constrained, Def, Expr, Lit, Span, Spanned, Typed};
use crate::passes::select::std_lib::Std;
use crate::passes::select::{Instr, VarArg};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use derive_more::Display;
use itertools::Itertools;
use partial_type::PartialType;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Display)]
#[display(fmt = "{}", r#"defs.values().format("\n")"#)]
pub struct PrgValidated<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefValidated<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

pub struct PrgConstrained<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefConstrained<'p>>,
    pub entry: UniqueSym<'p>,
    pub uf: UnionFind<PartialType<'p>>,
    pub std: Std<'p>,
}

pub type DefValidated<'p> = Def<UniqueSym<'p>, &'p str, Typed<'p, ExprValidated<'p>>>;
pub type ExprValidated<'p> = Expr<UniqueSym<'p>, &'p str, TLit, Type<UniqueSym<'p>>>;

pub type DefConstrained<'p> =
    Def<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Constrained<ExprConstrained<'p>>>;
pub type ExprConstrained<'p> =
    Expr<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Lit<'p>, MetaConstrained>;

pub type DefUniquified<'p> =
    Def<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Spanned<ExprUniquified<'p>>>;
pub type ExprUniquified<'p> = Expr<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Lit<'p>, Span>;
pub type InstrUniquified<'p> = Instr<VarArg<Spanned<UniqueSym<'p>>>, Spanned<UniqueSym<'p>>>;

pub struct MetaConstrained {
    pub span: Span,
    pub index: UnionIndex,
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum TLit {
    #[display(fmt = "{val}")]
    I64 { val: i64 },
    #[display(fmt = "{val}")]
    U64 { val: u64 },
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
        match self {
            TLit::I64 { val, .. } => val,
            TLit::U64 { val, .. } => val as i64,
            _ => panic!(),
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
            TLit::I64 { val } => val,
            TLit::U64 { val } => val as i64,
            TLit::Bool { val } => val as i64,
            TLit::Unit => 0,
        }
    }
}

impl FromStr for TLit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "false" => TLit::Bool { val: false },
            "true" => TLit::Bool { val: true },
            "unit" => TLit::Unit,
            s => TLit::I64 {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}
