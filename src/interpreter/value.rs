use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Val {
    Int { val: i64 },
    Bool { val: bool },
}

impl Val {
    pub fn int(self) -> i64 {
        match self {
            Val::Int { val } => val,
            Val::Bool { .. } => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { val } => val,
        }
    }
}

impl Into<i64> for Val {
    fn into(self) -> i64 {
        match self {
            Val::Int { val } => val,
            Val::Bool { val } => val as i64,
        }
    }
}

impl FromStr for Val {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "f" => Val::Bool { val: false },
            "t" => Val::Bool { val: true },
            s => Val::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}

impl Display for Val {
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
        }
    }
}
