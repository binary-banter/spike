use crate::language::lvar::{Expr, LVarProgram, ULVarProgram};
use crate::utils::push_map::PushMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

impl LVarProgram {
    pub fn uniquify(self) -> ULVarProgram {
        ULVarProgram {
            bdy: uniquify_expression(self.bdy, &mut PushMap::default()),
        }
    }
}

fn uniquify_expression(expr: Expr, scope: &mut PushMap<String, String>) -> Expr {
    match expr {
        Expr::Int { .. } => expr,
        Expr::Var { sym } => Expr::Var {
            sym: scope[&sym].clone(),
        },
        Expr::Prim { op, args } => Expr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
        },
        Expr::Let { sym, bnd, bdy } => {
            let unique_bnd = uniquify_expression(*bnd, scope);
            let unique_sym = gen_sym(&sym);
            let unique_bdy = scope.push(sym, unique_sym.clone(), |scope| {
                uniquify_expression(*bdy, scope)
            });

            Expr::Let {
                sym: unique_sym,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
    }
}

pub fn gen_sym(input: &str) -> String {
    format!("{input}_{}", COUNT.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use crate::interpreter::lvar::interpret_lvar;
    use crate::interpreter::TestIO;
    use crate::language::lvar::Expr;
    use crate::utils::split_test::split_test;
    use std::collections::HashSet;
    use test_each_file::test_each_file;

    fn unique([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let uniquified_program = program.uniquify().into();
        let mut io = TestIO::new(input);
        let result = interpret_lvar(&uniquified_program, &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
        assert_unique_expr(&uniquified_program.bdy, &mut HashSet::new());
    }

    fn assert_unique_expr(expr: &Expr, vars: &mut HashSet<String>) {
        match expr {
            Expr::Int { .. } | Expr::Var { .. } => {}
            Expr::Prim { args, .. } => args.iter().for_each(|arg| assert_unique_expr(arg, vars)),
            Expr::Let { sym, bnd, bdy } => {
                assert!(vars.insert(sym.clone()));
                assert_unique_expr(bnd, vars);
                assert_unique_expr(bdy, vars);
            }
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as uniquify => unique }
}
