use crate::alvar::{AExpr, Atom};
use crate::lvar::LVarProgram;
use crate::lvar::{ALVarProgram, Expr};
use crate::uniquify::gen_sym;

pub fn rco_program(program: LVarProgram) -> ALVarProgram {
    ALVarProgram {
        bdy: rco_expr(program.bdy),
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
                .fold(AExpr::Prim { op, args }, |bdy, (sym, bnd)| AExpr::Let {
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
    use crate::parser::parse_program;
    use crate::remove_complex_operands::rco_program;
    use crate::uniquify::uniquify_program;

    #[test]
    fn simple() {
        dbg!(rco_program(uniquify_program(
            parse_program("(+ 10 (let (x 32) x))").unwrap().1
        )));
    }

    #[test]
    fn complex() {
        dbg!(rco_program(uniquify_program(
            parse_program("(let (x (+ 1 (let (y 1) y))) x)").unwrap().1
        )));
    }
}
