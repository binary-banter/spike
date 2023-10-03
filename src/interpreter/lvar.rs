use crate::interpreter::IO;
use crate::language::lvar::{Expr, LVarProgram, Op};
use crate::utils::push_map::PushMap;

pub fn interpret_lvar(program: &LVarProgram, io: &mut impl IO) -> i64 {
    interpret_expr(&program.bdy, &mut PushMap::default(), io)
}

fn interpret_expr(expr: &Expr, scope: &mut PushMap<String, i64>, io: &mut impl IO) -> i64 {
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
    use crate::parser::parse_program;
    use test_each_file::test_each_file;

    fn interpret([test]: [&str; 1]) {
        let mut test = test.split("#");
        let input = test.next().unwrap().trim();
        let expected_output = test.next().unwrap().trim();
        let expected_return = test.next().unwrap().trim();
        let program = test.next().unwrap().trim();

        let program = parse_program(program).unwrap().1;
        let input = input
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.parse().unwrap())
            .collect();
        let expected_output: Vec<_> = expected_output
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.parse().unwrap())
            .collect();
        let expected_return = expected_return.trim().parse().unwrap();

        let mut testio = TestIO::new(input);
        let res = interpret_lvar(&program, &mut testio);

        assert_eq!(res, expected_return);
        assert_eq!(testio.outputs, expected_output);
    }

    test_each_file! { for ["test"] in "./programs/good" as interpreter => interpret }
}
