pub mod explicate;
pub mod interpret;

use crate::passes::atomize::Atom;
use crate::passes::parse::types::Type;
use crate::passes::parse::{Op, Param, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgExplicated<'p> {
    pub blocks: HashMap<UniqueSym<'p>, CTail<'p>>,
    pub fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    pub defs: HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum CTail<'p> {
    Return {
        expr: Atom<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        sym: UniqueSym<'p>,
        bnd: CExpr<'p>,
        tail: Box<CTail<'p>>,
    },
    IfStmt {
        cnd: CExpr<'p>,
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
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
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

impl<'p> CExpr<'p> {
    pub fn typ(&self) -> &Type<UniqueSym<'p>> {
        match self {
            CExpr::Atom { typ, .. }
            | CExpr::Prim { typ, .. }
            | CExpr::Apply { typ, .. }
            | CExpr::FunRef { typ, .. }
            | CExpr::Struct { typ, .. }
            | CExpr::AccessField { typ, .. } => typ,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn explicated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program.validate().unwrap().reveal().atomize().explicate();

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as explicate => explicated }
}
