use crate::language::lvar::Lit;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Val<A: Copy + Hash + Eq> {
    Int { val: i64 },
    Bool { val: bool },
    Function { sym: A },
}

impl<A: Copy + Hash + Eq> Val<A> {
    pub fn int(self) -> i64 {
        match self {
            Val::Int { val } => val,
            Val::Bool { .. } => panic!(),
            Val::Function { .. } => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { val } => val,
            Val::Function { .. } => panic!(),
        }
    }
}

impl From<Lit> for i64 {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => val,
            Lit::Bool { val } => val as i64,
        }
    }
}

impl FromStr for Lit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "f" => Lit::Bool { val: false },
            "t" => Lit::Bool { val: true },
            s => Lit::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}

impl<A: Copy + Hash + Eq + Display> Display for Val<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Val::Int { val } => write!(f, "{val}"),
            Val::Bool { val } => {
                if *val {
                    write!(f, "t")
                } else {
                    write!(f, "f")
                }
            }
            Val::Function { sym, .. } => write!(f, "pointer to `{sym}``"),
        }
    }
}
