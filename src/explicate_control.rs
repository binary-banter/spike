use crate::language::alvar::AExpr;
use crate::language::alvar::ALVarProgram;
use crate::language::cvar::CExpr;
use crate::language::cvar::{CVarProgram, Tail};

pub fn explicate_program(program: ALVarProgram) -> CVarProgram {
    CVarProgram {
        bdy: explicate_tail(program.bdy),
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

fn explicate_assign(sym: String, bnd: AExpr, tail: Tail) -> Tail {
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
    use crate::interpreter::lvar::interpret_lvar;
    use crate::interpreter::TestIO;
    use crate::remove_complex_operands::rco_program;
    use crate::uniquify::uniquify_program;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;
    use crate::explicate_control::explicate_program;

    fn explicated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = explicate_program(rco_program(uniquify_program(program)));
        let mut io = TestIO::new(input);
        let result = interpret_lvar(&program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as explicate => explicated }
}
