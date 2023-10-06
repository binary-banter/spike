use crate::language::lvar::{Expr, LVarProgram, Op, ULVarProgram};
use crate::passes::uniquify::UniqueSym;

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
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
        bdy: Box<AExpr<'p>>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Atom<'p> {
    Int { val: i64 },
    Var { sym: UniqueSym<'p> },
}

impl<'p> From<ALVarProgram<'p>> for ULVarProgram<'p> {
    fn from(value: ALVarProgram<'p>) -> Self {
        ULVarProgram {
            bdy: value.bdy.into(),
        }
    }
}

impl<'p> From<AExpr<'p>> for Expr< UniqueSym<'p>> {
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

impl<'p> From<Atom<'p>> for Expr< UniqueSym<'p>> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Int { val } => Expr::Int { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
