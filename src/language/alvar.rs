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

impl Into<LVarProgram> for ALVarProgram {
    fn into(self) -> LVarProgram {
        LVarProgram {
            bdy: self.bdy.into(),
        }
    }
}

impl Into<Expr> for AExpr {
    fn into(self) -> Expr {
        match self {
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

impl Into<Expr> for Atom {
    fn into(self) -> Expr {
        match self {
            Atom::Int { val } => Expr::Int { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
