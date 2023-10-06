use crate::language::lvar::{Expr, LVarProgram, Op};

#[derive(Debug, PartialEq)]
pub struct ALVarProgram<'p> {
    pub bdy: AExpr<'p>,
}

#[derive(Debug, PartialEq)]
pub enum AExpr<'p> {
    Atom(Atom<'p>),
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
    },
    Let {
        sym: &'p str,
        bnd: Box<AExpr<'p>>,
        bdy: Box<AExpr<'p>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Atom<'p> {
    Int { val: i64 },
    Var { sym: &'p str },
}

impl<'p> From<ALVarProgram<'p>> for LVarProgram<'p> {
    fn from(value: ALVarProgram<'p>) -> Self {
        LVarProgram {
            bdy: value.bdy.into(),
        }
    }
}

impl<'p> From<AExpr<'p>> for Expr<'p> {
    fn from(value: AExpr<'p>) -> Self {
        match value {
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

impl<'p> From<Atom<'p>> for Expr<'p> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Int { val } => Expr::Int { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
