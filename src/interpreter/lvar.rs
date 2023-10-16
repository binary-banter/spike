use crate::interpreter::value::Val;
use crate::interpreter::IO;
use crate::language::lvar::{Expr, Op, SLVarProgram};
use crate::utils::push_map::PushMap;
use std::hash::Hash;

impl<A: Copy + Hash + Eq> SLVarProgram<A> {
    pub fn interpret(&self, io: &mut impl IO) -> Val {
        todo!()
        // interpret_expr(&self.bdy, &mut PushMap::default(), io)
    }
}

pub fn interpret_expr<A: Copy + Hash + Eq>(
    expr: &Expr<A>,
    scope: &mut PushMap<A, Val>,
    io: &mut impl IO,
) -> Val {
    match expr {
        Expr::Val { val } => *val,
        Expr::Var { sym } => scope[sym],
        Expr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Read, []) => io.read(),
            (Op::Print, [v]) => {
                let v = interpret_expr(v, scope, io);
                io.print(v);
                v
            }
            (Op::Plus, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Int { val: e1 + e2 }
            }
            (Op::Minus, [e1]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                Val::Int { val: -e1 }
            }
            (Op::Minus, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Int { val: e1 - e2 }
            }
            (Op::Mul, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Int { val: e1 * e2 }
            }
            (Op::GT, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Bool { val: e1 > e2 }
            }
            (Op::GE, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Bool { val: e1 >= e2 }
            }
            (Op::LT, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Bool { val: e1 < e2 }
            }
            (Op::LE, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).int();
                let e2 = interpret_expr(e2, scope, io).int();
                Val::Bool { val: e1 <= e2 }
            }
            (Op::EQ, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io);
                let e2 = interpret_expr(e2, scope, io);
                Val::Bool { val: e1 == e2 }
            }
            (Op::NE, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io);
                let e2 = interpret_expr(e2, scope, io);
                Val::Bool { val: e1 != e2 }
            }
            (Op::Not, [e1]) => {
                let e1 = interpret_expr(e1, scope, io).bool();
                Val::Bool { val: !e1 }
            }
            (Op::LAnd, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).bool();
                if !e1 {
                    return Val::Bool { val: false };
                }
                interpret_expr(e2, scope, io)
            }
            (Op::LOr, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).bool();
                if e1 {
                    return Val::Bool { val: true };
                }
                interpret_expr(e2, scope, io)
            }
            (Op::Xor, [e1, e2]) => {
                let e1 = interpret_expr(e1, scope, io).bool();
                let e2 = interpret_expr(e2, scope, io).bool();
                Val::Bool { val: e1 ^ e2 }
            }
            _ => unreachable!(),
        },
        Expr::Let { sym, bnd, bdy } => {
            let bnd = interpret_expr(bnd, scope, io);
            scope.push(*sym, bnd, |scope| interpret_expr(bdy, scope, io))
        }
        Expr::If { cnd, thn, els } => {
            if interpret_expr(cnd, scope, io).bool() {
                interpret_expr(thn, scope, io)
            } else {
                interpret_expr(els, scope, io)
            }
        }
        Expr::Apply { .. } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn interpret([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return);
        assert_eq!(io.outputs, expected_output);
    }

    test_each_file! { for ["test"] in "./programs/good" as interpreter => interpret }
}
