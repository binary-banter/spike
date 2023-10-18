use crate::interpreter::value::Val;
use crate::interpreter::IO;
use crate::language::lvar::{Def, Expr, Lit, Op, PrgGenericVar};
use crate::utils::push_map::PushMap;
use std::hash::Hash;

impl<A: Copy + Hash + Eq> PrgGenericVar<A> {
    pub fn interpret(&self, io: &mut impl IO) -> Val<A> {
        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|(&sym, _)| (sym, Val::Function { sym })),
        );
        self.interpret_fn(self.entry, Vec::new(), &mut scope, io)
    }

    fn interpret_fn(
        &self,
        sym: A,
        args: Vec<Val<A>>,
        scope: &mut PushMap<A, Val<A>>,
        io: &mut impl IO,
    ) -> Val<A> {
        match &self.defs[&sym] {
            Def::Fn { prms, bdy, .. } => scope.push_iter(
                prms.iter().zip(args.iter()).map(|((k, _), v)| (*k, *v)),
                |scope| self.interpret_expr(bdy, scope, io),
            ),
        }
    }

    pub fn interpret_expr(
        &self,
        expr: &Expr<A>,
        scope: &mut PushMap<A, Val<A>>,
        io: &mut impl IO,
    ) -> Val<A> {
        match expr {
            Expr::Lit { val } => (*val).into(),
            Expr::Var { sym } => scope[sym],
            Expr::Prim { op, args } => match (op, args.as_slice()) {
                (Op::Read, []) => io.read().into(),
                (Op::Print, [v]) => {
                    let val = self.interpret_expr(v, scope, io);
                    io.print(Lit::Int { val: val.int() });
                    val
                }
                (Op::Plus, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Int { val: e1 + e2 }
                }
                (Op::Minus, [e1]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    Val::Int { val: -e1 }
                }
                (Op::Minus, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Int { val: e1 - e2 }
                }
                (Op::Mul, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Int { val: e1 * e2 }
                }
                (Op::GT, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Bool { val: e1 > e2 }
                }
                (Op::GE, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Bool { val: e1 >= e2 }
                }
                (Op::LT, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Bool { val: e1 < e2 }
                }
                (Op::LE, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).int();
                    let e2 = self.interpret_expr(e2, scope, io).int();
                    Val::Bool { val: e1 <= e2 }
                }
                (Op::EQ, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io);
                    let e2 = self.interpret_expr(e2, scope, io);
                    Val::Bool { val: e1 == e2 }
                }
                (Op::NE, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io);
                    let e2 = self.interpret_expr(e2, scope, io);
                    Val::Bool { val: e1 != e2 }
                }
                (Op::Not, [e1]) => {
                    let e1 = self.interpret_expr(e1, scope, io).bool();
                    Val::Bool { val: !e1 }
                }
                (Op::LAnd, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).bool();
                    if !e1 {
                        return Val::Bool { val: false };
                    }
                    self.interpret_expr(e2, scope, io)
                }
                (Op::LOr, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).bool();
                    if e1 {
                        return Val::Bool { val: true };
                    }
                    self.interpret_expr(e2, scope, io)
                }
                (Op::Xor, [e1, e2]) => {
                    let e1 = self.interpret_expr(e1, scope, io).bool();
                    let e2 = self.interpret_expr(e2, scope, io).bool();
                    Val::Bool { val: e1 ^ e2 }
                }
                _ => unreachable!(),
            },
            Expr::Let { sym, bnd, bdy } => {
                let bnd = self.interpret_expr(bnd, scope, io);
                scope.push(*sym, bnd, |scope| self.interpret_expr(bdy, scope, io))
            }
            Expr::If { cnd, thn, els } => {
                if self.interpret_expr(cnd, scope, io).bool() {
                    self.interpret_expr(thn, scope, io)
                } else {
                    self.interpret_expr(els, scope, io)
                }
            }
            Expr::Apply { fun, args } => {
                let sym = self.interpret_expr(fun, scope, io).fun();
                let args = args
                    .iter()
                    .map(|arg| self.interpret_expr(arg, scope, io))
                    .collect();
                self.interpret_fn(sym, args, scope, io)
            }
        }
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
        let result = program.type_check().unwrap().interpret(&mut io);

        assert_eq!(result, expected_return.into());
        assert_eq!(io.outputs, expected_output);
    }

    test_each_file! { for ["test"] in "./programs/good" as interpreter => interpret }
}
