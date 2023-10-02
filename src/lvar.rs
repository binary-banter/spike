#[derive(Debug, PartialEq)]
pub struct LVarProgram {
    pub bdy: Expr,
}

#[allow(unused)]
pub enum Type {
    Integer,
}

#[derive(Debug, PartialEq)]
pub enum Operation {
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
        op: Operation,
        args: Vec<Expr>,
    },
    Let {
        sym: String,
        bnd: Box<Expr>,
        bdy: Box<Expr>,
    },
}
