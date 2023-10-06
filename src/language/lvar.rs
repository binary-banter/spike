#[derive(Debug, PartialEq)]
pub struct LVarProgram<'p> {
    pub bdy: Expr<'p>,
}

#[derive(Debug, PartialEq)]
pub struct ULVarProgram<'p> {
    pub bdy: Expr<'p>,
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq)]
pub enum Expr<'p> {
    Int {
        val: i64,
    },
    Var {
        sym: &'p str,
    },
    Prim {
        op: Op,
        args: Vec<Expr<'p>>,
    },
    Let {
        sym: &'p str,
        bnd: Box<Expr<'p>>,
        bdy: Box<Expr<'p>>,
    },
}

impl<'p> From<ULVarProgram<'p>> for LVarProgram<'p> {
    fn from(value: ULVarProgram<'p>) -> Self {
        LVarProgram { bdy: value.bdy }
    }
}
