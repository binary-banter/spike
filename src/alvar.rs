use crate::lvar::Op;

#[derive(Debug, PartialEq)]
pub enum AExpr {
    Atom(Atom),
    Prim {
        op: Op,
        args: Vec<Atom>,
    },
    Let {
        sym: String,
        bnd: Box<AExpr>,
        bdy: Box<AExpr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Atom {
    Int { val: i64 },
    Var { sym: String },
}
