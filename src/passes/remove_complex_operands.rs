use crate::language::alvar::{AExpr, ALVarProgram, Atom};
use crate::language::lvar::{Expr, ULVarProgram};
use crate::passes::uniquify::gen_sym;

impl ULVarProgram {
    pub fn remove_complex_operands(self) -> ALVarProgram {
        ALVarProgram {
            bdy: rco_expr(self.bdy),
        }
    }
}

fn rco_expr(expr: Expr) -> AExpr {
    match expr {
        Expr::Int { val } => AExpr::Atom(Atom::Int { val }),
        Expr::Var { sym } => AExpr::Atom(Atom::Var { sym }),
        Expr::Prim { op, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(rco_atom).unzip();

            extras
                .into_iter()
                .flatten()
                .rfold(AExpr::Prim { op, args }, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
        }
        Expr::Let { sym, bnd, bdy } => AExpr::Let {
            sym,
            bnd: Box::new(rco_expr(*bnd)),
            bdy: Box::new(rco_expr(*bdy)),
        },
    }
}

fn rco_atom(expr: Expr) -> (Atom, Option<(String, AExpr)>) {
    match expr {
        Expr::Int { val } => (Atom::Int { val }, None),
        Expr::Var { sym } => (Atom::Var { sym }, None),
        Expr::Prim { .. } | Expr::Let { .. } => {
            let tmp = gen_sym("");
            (Atom::Var { sym: tmp.clone() }, Some((tmp, rco_expr(expr))))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::lvar::interpret_lvar;
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn atomic([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program.uniquify().remove_complex_operands();
        let mut io = TestIO::new(input);
        let result = interpret_lvar(&program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as remove_complex_operands => atomic }
}
