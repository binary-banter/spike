pub mod eliminate;
mod eliminate_params;

use crate::passes::atomize::Atom;
use crate::passes::explicate::{CExpr, PrgExplicated, CTail};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Op, Param, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgEliminated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, ETail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum ETail<'p> {
    Return {
        expr: Vec<Atom<'p>>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: EExpr<'p>,
        tail: Box<ETail<'p>>,
    },
    IfStmt {
        cnd: EExpr<'p>,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
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
        typ: Vec<Type<UniqueSym<'p>>>,
        fn_typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
}

// todo: we need a new interpreter
// #[cfg(test)]
// mod tests {
//     use crate::interpreter::TestIO;
//     use crate::passes::explicate::PrgExplicated;
//     use crate::utils::split_test::split_test;
//     use test_each_file::test_each_file;
//
//     fn eliminated([test]: [&str; 1]) {
//         let (input, expected_output, expected_return, program) = split_test(test);
//         let program: PrgExplicated = program
//             .validate()
//             .unwrap()
//             .uniquify()
//             .reveal()
//             .atomize()
//             .explicate()
//             .eliminate()
//             .into();
//
//         let mut io = TestIO::new(input);
//
//         let result = program.interpret(&mut io);
//
//         assert_eq!(result, expected_return.into(), "Incorrect program result.");
//         assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
//     }
//
//     test_each_file! { for ["test"] in "./programs/good" as eliminate_algebraic => eliminated }
// }
