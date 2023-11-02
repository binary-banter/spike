pub mod explicate;
pub mod interpret;

use crate::passes::atomize::Atom;
use crate::passes::parse::Op;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Tail<'p, CExpr<'p>>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<UniqueSym<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum Tail<'p, E> {
    Return {
        expr: E,
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
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<Atom<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Struct {
        sym: UniqueSym<'p>,
        // todo: this does not need to be atom!
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
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
