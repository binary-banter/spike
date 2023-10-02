use crate::lvar::Expr;

#[derive(Debug, PartialEq)]
pub struct CVarProgram {
    pub blocks: Vec<(String, Tail)>,
}

#[derive(Debug, PartialEq)]
pub enum Tail {
    Return {
        expr: Expr,
    },
    Seq {
        sym: String,
        bnd: Expr,
        tail: Box<Tail>,
    },
}
