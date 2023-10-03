#[derive(Debug, PartialEq)]
pub struct LVarProgram {
    pub bdy: Expr,
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int {
        val: i64,
    },
    Var {
        sym: String,
    },
    Prim {
        op: Op,
        args: Vec<Expr>,
    },
    Let {
        sym: String,
        bnd: Box<Expr>,
        bdy: Box<Expr>,
    },
}
