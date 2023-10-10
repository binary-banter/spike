use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Value {
    Int{ val: i64},
    Bool{ val: bool },
}

impl Into<i64> for Value {
    fn into(self) -> i64 {
        match self {
            Value::Int { val } => val,
            Value::Bool { val } => val as i64,
        }
    }
}

impl FromStr for Value {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "f" => Value::Bool{ val: false},
            "t" => Value::Bool{ val: true},
            s => Value::Int { val: s.parse().map_err(|_| ())? }
        })
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int { val } => write!(f, "{val}"),
            Value::Bool { val } => if *val {write!(f, "t")} else {write!(f, "f")},
        }
    }
}
