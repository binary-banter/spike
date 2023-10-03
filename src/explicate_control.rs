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
    use crate::explicate_control::explicate_program;
    use crate::parser::parse_program;
    use crate::remove_complex_operands::rco_program;
    use crate::uniquify::uniquify_program;

    #[test]
    fn simple() {
        dbg!(explicate_program(rco_program(uniquify_program(
            parse_program("(+ 10 (let (x 32) x))").unwrap().1
        ))));
    }

    #[test]
    fn complex() {
        dbg!(explicate_program(rco_program(uniquify_program(
            parse_program("(let (x (+ 1 (let (y 1) y))) x)").unwrap().1
        ))));
    }
}