use crate::parser::Expr;
use crate::LVarProgram;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn uniquify_program(program: LVarProgram) -> LVarProgram {
    LVarProgram {
        bdy: uniquify_expression(program.bdy, &mut HashMap::new()),
    }
}

fn uniquify_expression(expr: Expr, scope: &mut HashMap<String, String>) -> Expr {
    match expr {
        Expr::Int(_) => expr,
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
            scope.insert(sym.clone(), unique_sym.clone());
            let unique_bdy = uniquify_expression(*bdy, scope);
            scope.remove(&sym);
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
    use crate::parser::parse_program;
    use crate::uniquify::uniquify_program;

    #[test]
    fn simple() {
        dbg!(uniquify_program(parse_program("(let (x 1) 1)").unwrap().1));
    }

    #[test]
    fn double_let_with_shadowing() {
        dbg!(uniquify_program(
            parse_program("(let (x 1) (let (x x) 1))").unwrap().1
        ));
    }
}
