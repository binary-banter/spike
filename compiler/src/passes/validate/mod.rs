mod check_sized;
pub mod error;
mod generate_constraints;
mod resolve_types;
mod solve_constraints;
#[cfg(test)]
mod tests;
mod uncover_globals;
pub mod uniquify;
pub mod validate;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr, Meta, Op, Span};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::UnionIndex;
use derive_more::Display;
use std::collections::HashMap;
use std::str::FromStr;

pub struct PrgValidated<'p> {
    pub defs: HashMap<&'p str, Def<UniqueSym<'p>, &'p str, TExpr<'p>>>,
    pub entry: &'p str,
}

pub struct PrgConstrained<'p> {
    pub defs: HashMap<UniqueSym<'p>, DefConstrained<'p>>,
    pub entry: UniqueSym<'p>,
}

pub type DefConstrained<'p> =
    Def<Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>, Meta<CMeta, ExprConstrained<'p>>>;
pub type ExprConstrained<'p> = Expr<'p, Meta<Span, UniqueSym<'p>>, Meta<Span, &'p str>, CMeta>;

pub struct CMeta {
    pub span: Span,
    pub index: UnionIndex,
}

// todo: burn this with fire and make it regular expressions (not regex tho haha)
pub enum TExpr<'p> {
    Lit {
        val: TLit,
        typ: Type<UniqueSym<'p>>,
    },
    Var {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: Op,
        args: Vec<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<TExpr<'p>>,
        bdy: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    If {
        cnd: Box<TExpr<'p>>,
        thn: Box<TExpr<'p>>,
        els: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Box<TExpr<'p>>,
        args: Vec<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Loop {
        bdy: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Break {
        bdy: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Continue {
        typ: Type<UniqueSym<'p>>,
    },
    Return {
        bdy: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        stmt: Box<TExpr<'p>>,
        cnt: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, TExpr<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    Variant {
        enum_sym: UniqueSym<'p>,
        variant_sym: &'p str,
        bdy: Box<TExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Box<TExpr<'p>>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
    Switch {
        enm: Box<TExpr<'p>>,
        arms: Vec<(UniqueSym<'p>, &'p str, Box<TExpr<'p>>)>,
        typ: Type<UniqueSym<'p>>,
    },
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

impl<'p> TExpr<'p> {
    pub fn typ(&self) -> &Type<UniqueSym<'p>> {
        match self {
            TExpr::Lit { typ, .. }
            | TExpr::Var { typ, .. }
            | TExpr::Prim { typ, .. }
            | TExpr::Let { typ, .. }
            | TExpr::If { typ, .. }
            | TExpr::Apply { typ, .. }
            | TExpr::Loop { typ, .. }
            | TExpr::Break { typ, .. }
            | TExpr::Continue { typ, .. }
            | TExpr::Return { typ, .. }
            | TExpr::Seq { typ, .. }
            | TExpr::Assign { typ, .. }
            | TExpr::Struct { typ, .. }
            | TExpr::Variant { typ, .. }
            | TExpr::AccessField { typ, .. }
            | TExpr::Switch { typ, .. } => typ,
        }
    }
}
