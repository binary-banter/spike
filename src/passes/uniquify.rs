//! This pass deals with the shadowing of variables by renaming every variable to a unique name.
//! The names need to be globally unique, not just in their scope.
//! This is useful because in later passes we will be changing the structure of the program,
//! and after selecting instructions we will only have a list of X86 instructions left.

use crate::language::lvar::{Expr, LVarProgram, ULVarProgram};
use crate::utils::push_map::PushMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

impl<'p> LVarProgram<'p> {
    //! See module-level documentation.
    pub fn uniquify(self) -> ULVarProgram<'p> {
        ULVarProgram {
            bdy: uniquify_expression(self.bdy, &mut PushMap::default()),
        }
    }
}

fn uniquify_expression<'p>(
    expr: Expr<&'p str>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Expr<UniqueSym<'p>> {
    match expr {
        Expr::Int { val } => Expr::Int { val },
        Expr::Var { sym } => Expr::Var { sym: scope[&sym] },
        Expr::Prim { op, args } => Expr::Prim {
            op,
            args: args
                .into_iter()
                .map(|arg| uniquify_expression(arg, scope))
                .collect(),
        },
        Expr::Let { sym, bnd, bdy } => {
            let unique_bnd = uniquify_expression(*bnd, scope);
            let unique_sym = gen_sym(sym);
            let unique_bdy = scope.push(sym, unique_sym, |scope| uniquify_expression(*bdy, scope));

            Expr::Let {
                sym: unique_sym,
                bnd: Box::new(unique_bnd),
                bdy: Box::new(unique_bdy),
            }
        }
        Expr::If { .. } => todo!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub struct UniqueSym<'p> {
    pub sym: &'p str,
    pub id: usize,
}

pub fn gen_sym(sym: &str) -> UniqueSym<'_> {
    UniqueSym {
        sym,
        id: COUNT.fetch_add(1, Ordering::Relaxed),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn unique([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let uniquified_program = program.uniquify();
        let mut io = TestIO::new(input);
        let result = uniquified_program.interpret(&mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as uniquify => unique }
}
