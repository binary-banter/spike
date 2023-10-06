use crate::interpreter::IO;
use crate::language::lvar::{Expr, LVarProgram, Op};
use crate::utils::push_map::PushMap;

pub fn interpret_lvar<'p>(program: &LVarProgram<'p>, io: &mut impl IO) -> i64 {
    interpret_expr(&program.bdy, &mut PushMap::default(), io)
}

fn interpret_expr<'p>(expr: &Expr<'p>, scope: &mut PushMap<&'p str, i64>, io: &mut impl IO) -> i64 {
    match expr {
        Expr::Int { val } => *val,
        Expr::Var { sym } => scope[sym],
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Read, []) => io.read(),
            (Op::Print, [v]) => {
                let v = interpret_expr(v, scope, io);
                io.print(v);
                v
            }
            (Op::Plus, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io);
                let e2 = interpret_expr(e2, scope, io);
                e1 + e2
            }
            (Op::Minus, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io);
                let e2 = interpret_expr(e2, scope, io);
                e1 - e2
            }
            (Op::Minus, [e1]) => {
                let e1 = interpret_expr(e1, scope, io);
                -e1
            }
            _ => unreachable!(),
        },
        Expr::Let { sym, bnd, bdy } => {
            let bnd = interpret_expr(bnd, scope, io);
            scope.push(sym.clone(), bnd, |scope| interpret_expr(bdy, scope, io))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::lvar::interpret_lvar;
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn interpret([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);

        let mut testio = TestIO::new(input);
        let res = interpret_lvar(&program, &mut testio);

        assert_eq!(res, expected_return);
        assert_eq!(testio.outputs, expected_output);
    }

    test_each_file! { for ["test"] in "./programs/good" as interpreter => interpret }
}
