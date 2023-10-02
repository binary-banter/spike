use crate::alvar::Atom;
use crate::lvar::Op;

#[derive(Debug, PartialEq)]
pub enum EExpr {
    Atom(Atom),
    Prim { op: Op, args: Vec<Atom> },
}
