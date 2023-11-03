pub mod explicate;
pub mod interpret;

use crate::passes::atomize::{AExpr, Atom};
use crate::passes::parse::{Def, Op, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::parse::types::Type;

#[derive(Debug, PartialEq)]
pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Tail<'p, CExpr<'p>>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<UniqueSym<'p>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum Tail<'p, E> {
    Return {
        expr: Atom<'p>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: E,
        tail: Box<Tail<'p, E>>,
    },
    IfStmt {
        cnd: E,
        thn: UniqueSym<'p>,
        els: UniqueSym<'p>,
    },
    Goto {
        lbl: UniqueSym<'p>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CExpr<'p> {
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
    Struct {
        sym: UniqueSym<'p>,
        // todo: this does not need to be atom!
        fields: Vec<(&'p str, Atom<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn explicated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .explicate();

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as explicate => explicated }
}
