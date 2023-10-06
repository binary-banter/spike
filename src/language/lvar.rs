use crate::passes::uniquify::UniqueSym;

pub type LVarProgram<'p> = GLVarProgram<&'p str>;
pub type ULVarProgram<'p> = GLVarProgram<UniqueSym<'p>>;

#[derive(Debug, PartialEq)]
pub struct GLVarProgram<A> {
    pub bdy: Expr<A>,
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
