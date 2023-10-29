use crate::passes::parse::Lit;
use derive_more::Display;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Display)]
pub enum Val<A: Copy + Hash + Eq + Display> {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "fn pointer `{sym}`")]
    Function { sym: A },
}

impl<A: Copy + Hash + Eq + Display> From<Lit> for Val<A> {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => Val::Int { val },
            Lit::Bool { val } => Val::Bool { val },
            Lit::Unit => Val::Unit,
        }
    }
}

impl<A: Copy + Hash + Eq + Display> Val<A> {
    pub fn int(self) -> i64 {
        match self {
            Val::Int { val } => val,
            Val::Bool { .. } => panic!(),
            Val::Function { .. } => panic!(),
            Val::Unit => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { val } => val,
            Val::Function { .. } => panic!(),
            Val::Unit => panic!(),
        }
    }

    pub fn fun(self) -> A {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { .. } => panic!(),
            Val::Function { sym } => sym,
            Val::Unit => panic!(),
        }
    }
}

impl From<Lit> for i64 {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => val,
            Lit::Bool { val } => val as i64,
            Lit::Unit => 0,
        }
    }
}

impl FromStr for Lit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "false" => Lit::Bool { val: false },
            "true" => Lit::Bool { val: true },
            "unit" => Lit::Unit,
            s => Lit::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}
