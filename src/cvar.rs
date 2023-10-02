use crate::elvar::EExpr;

#[derive(Debug, PartialEq)]
pub struct CVarProgram {
    pub blocks: Vec<(String, Tail)>,
}

#[derive(Debug, PartialEq)]
pub enum Tail {
    Return {
        expr: EExpr,
    },
    Seq {
        sym: String,
        bnd: EExpr,
        tail: Box<Tail>,
    },
}
