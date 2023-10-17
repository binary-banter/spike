use std::collections::HashMap;
use crate::language::lvar::{Expr, Lit, Op, PrgGenericVar};
use crate::passes::uniquify::UniqueSym;
use crate::passes::type_check::Type;

#[derive(Debug, PartialEq)]
pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, ADef<'p>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum ADef<'p> {
    Fn {
        sym: UniqueSym<'p>,
        prms: Vec<(UniqueSym<'p>, Type)>,
        typ: Type,
        bdy: AExpr<'p>,
    },
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
    If {
        cnd: Box<AExpr<'p>>,
        thn: Box<AExpr<'p>>,
        els: Box<AExpr<'p>>,
    },
    Apply {
        fun: Box<AExpr<'p>>,
        args: Vec<AExpr<'p>>,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Atom<'p> {
    Val { val: Lit },
    Var { sym: UniqueSym<'p> },
}

impl<'p> From<PrgAtomized<'p>> for PrgGenericVar<UniqueSym<'p>> {
    fn from(value: PrgAtomized<'p>) -> Self {
        todo!()
        // ULVarProgram {
        //     defs: todo!(),
        //     bdy: value.bdy.into(),
        // }
    }
}

impl<'p> From<AExpr<'p>> for Expr<UniqueSym<'p>> {
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
            AExpr::If { cnd, thn, els } => Expr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            AExpr::Apply { fun, args } => Expr::Apply {
                    fun: Box::new((*fun).into()),
                args: args.into_iter().map(Into::into).collect(),
            },
        }
    }
}

impl<'p> From<Atom<'p>> for Expr<UniqueSym<'p>> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Val { val } => Expr::Lit { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}
