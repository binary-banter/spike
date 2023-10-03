use crate::language::lvar::{Expr, LVarProgram, Op};

#[derive(Debug, PartialEq)]
pub struct ALVarProgram {
    pub bdy: AExpr,
}

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

impl From<ALVarProgram> for LVarProgram {
    fn from(val: ALVarProgram) -> Self {
        LVarProgram {
            bdy: val.bdy.into(),
        }
    }
}

impl From<AExpr> for Expr {
    fn from(val: AExpr) -> Self {
        match val {
            AExpr::Atom(a) => a.into(),
            AExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
            AExpr::Let { sym, bnd, bdy } => Expr::Let {
                sym,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
        }
    }
}

impl From<Atom> for Expr {
    fn from(val: Atom) -> Self {
        match val {
            Atom::Int { val } => Expr::Int { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
