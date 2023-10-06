use crate::passes::uniquify::UniqueSym;

#[derive(Debug, PartialEq)]
pub struct LVarProgram<'p> {
    pub bdy: Expr<&'p str>,
}

#[derive(Debug, PartialEq)]
pub struct ULVarProgram<'p> {
    pub bdy: Expr<UniqueSym<'p>>,
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Expr<A> {
    Int {
        val: i64,
    },
    Var {
        sym: A,
    },
    Prim {
        op: Op,
        args: Vec<Expr<A>>,
    },
    Let {
        sym: A,
        bnd: Box<Expr<A>>,
        bdy: Box<Expr<A>>,
    },
}
