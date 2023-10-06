use crate::language::alvar::{AExpr, ALVarProgram};
use crate::language::cvar::{CExpr, CVarProgram, Tail};
use crate::passes::uniquify::UniqueSym;

impl<'p> ALVarProgram<'p> {
    pub fn explicate(self) -> CVarProgram<'p> {
        CVarProgram {
            bdy: explicate_tail(self.bdy),
        }
    }
}

fn explicate_tail(expr: AExpr) -> Tail {
    match expr {
        AExpr::Atom(atom) => Tail::Return {
            expr: CExpr::Atom(atom),
        },
        AExpr::Prim { op, args } => Tail::Return {
            expr: CExpr::Prim { op, args },
        },
        AExpr::Let { sym, bnd, bdy } => explicate_assign(sym, *bnd, explicate_tail(*bdy)),
    }
}

fn explicate_assign<'p>(sym: UniqueSym<'p>, bnd: AExpr<'p>, tail: Tail<'p>) -> Tail<'p> {
    match bnd {
        AExpr::Atom(atom) => Tail::Seq {
            sym,
            bnd: CExpr::Atom(atom),
            tail: Box::new(tail),
        },
        AExpr::Prim { op, args } => Tail::Seq {
            sym,
            bnd: CExpr::Prim { op, args },
            tail: Box::new(tail),
        },
        AExpr::Let {
            sym: sym_,
            bnd: bnd_,
            bdy: bdy_,
        } => explicate_assign(sym_, *bnd_, explicate_assign(sym, *bdy_, tail)),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;
    use crate::language::lvar::ULVarProgram;

    fn explicated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program.uniquify().remove_complex_operands().explicate();
        let program: ULVarProgram = program.into();

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as explicate => explicated }
}
