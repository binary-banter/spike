pub mod eliminate;

use crate::passes::atomize::Atom;
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Op, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgEliminated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Tail<'p, EExpr<'p>>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<UniqueSym<'p>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EExpr<'p> {
    Atom {
        atm: Atom<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<Atom<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
}

impl<'p> From<PrgEliminated<'p>> for PrgExplicated<'p> {
    fn from(value: PrgEliminated<'p>) -> Self {
        PrgExplicated {
            blocks: value.blocks.fmap(From::from),
            fn_params: value.fn_params,
            defs: Default::default(),
            entry: value.entry,
        }
    }
}

impl<'p> From<Tail<'p, EExpr<'p>>> for Tail<'p, CExpr<'p>> {
    fn from(value: Tail<'p, EExpr<'p>>) -> Self {
        match value {
            Tail::Return { expr } => Tail::Return { expr },
            Tail::Seq { sym, bnd, tail } => Tail::Seq {
                sym,
                bnd: bnd.into(),
                tail: Box::new((*tail).into()),
            },
            Tail::IfStmt { cnd, thn, els } => Tail::IfStmt {
                cnd: cnd.into(),
                thn,
                els,
            },
            Tail::Goto { lbl } => Tail::Goto { lbl },
        }
    }
}

impl<'p> From<EExpr<'p>> for CExpr<'p> {
    fn from(value: EExpr<'p>) -> Self {
        match value {
            EExpr::Atom { atm, typ } => CExpr::Atom { atm, typ },
            EExpr::Prim { op, args, typ } => CExpr::Prim { op, args, typ },
            EExpr::Apply { fun, args, typ } => CExpr::Apply { fun, args, typ },
            EExpr::FunRef { sym, typ } => CExpr::FunRef { sym, typ },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::passes::explicate::PrgExplicated;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn eliminated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: PrgExplicated = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .explicate()
            .eliminate()
            .into();

        let mut io = TestIO::new(input);

        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as eliminate_algebraic => eliminated }
}
