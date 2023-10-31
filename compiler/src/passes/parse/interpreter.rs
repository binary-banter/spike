use crate::interpreter::Val;
use crate::interpreter::IO;
use crate::passes::parse::{Def, Expr, Lit, Op, PrgGenericVar};
use crate::utils::push_map::PushMap;
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(Copy, Clone)]
pub enum ControlFlow<A: Copy + Hash + Eq + Display> {
    Val(Val<A>),
    Break(Val<A>),
    Return(Val<A>),
    Continue,
}

/// This macro unwraps values and bubbles up continues, breaks and returns.
macro_rules! b {
    ($e: expr) => {{
        let e = $e;
        match e {
            ControlFlow::Val(val) => val,
            ControlFlow::Break(_) | ControlFlow::Return(_) | ControlFlow::Continue => return e,
        }
    }};
}

impl<A: Copy + Hash + Eq + Debug + Display> PrgGenericVar<A> {
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
            Def::Fn { params, bdy, .. } => scope.push_iter(
                params
                    .iter()
                    .zip(args.iter())
                    .map(|(param, v)| (param.sym, *v)),
                |scope| match self.interpret_expr(bdy, scope, io) {
                    ControlFlow::Return(val) | ControlFlow::Val(val) => val,
                    ControlFlow::Continue | ControlFlow::Break(_) => unreachable!(),
                },
            ),
            Def::Struct { .. } => todo!(),
            Def::Enum { .. } => todo!(),
        }
    }

    pub fn interpret_expr(
        &self,
        expr: &Expr<A>,
        scope: &mut PushMap<A, Val<A>>,
        io: &mut impl IO,
    ) -> ControlFlow<A> {
        ControlFlow::Val(match expr {
            Expr::Lit { val } => (*val).into(),
            Expr::Var { sym } => scope[sym],
            Expr::Prim { op, args } => match (op, args.as_slice()) {
                (Op::Read, []) => io.read().into(),
                (Op::Print, [v]) => {
                    let val = b!(self.interpret_expr(v, scope, io));
                    io.print(Lit::Int { val: val.int() });
                    val
                }
                (Op::Plus, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Int { val: e1 + e2 }
                }
                (Op::Minus, [e1]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    Val::Int { val: -e1 }
                }
                (Op::Minus, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Int { val: e1 - e2 }
                }
                (Op::Mul, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Int { val: e1 * e2 }
                }
                (Op::Div, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Int { val: e1 / e2 }
                }
                (Op::Mod, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Int { val: e1 % e2 }
                }
                (Op::GT, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Bool { val: e1 > e2 }
                }
                (Op::GE, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Bool { val: e1 >= e2 }
                }
                (Op::LT, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Bool { val: e1 < e2 }
                }
                (Op::LE, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).int();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).int();
                    Val::Bool { val: e1 <= e2 }
                }
                (Op::EQ, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io));
                    let e2 = b!(self.interpret_expr(e2, scope, io));
                    Val::Bool { val: e1 == e2 }
                }
                (Op::NE, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io));
                    let e2 = b!(self.interpret_expr(e2, scope, io));
                    Val::Bool { val: e1 != e2 }
                }
                (Op::Not, [e1]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).bool();
                    Val::Bool { val: !e1 }
                }
                (Op::LAnd, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).bool();
                    if !e1 {
                        return ControlFlow::Val(Val::Bool { val: false });
                    }
                    b!(self.interpret_expr(e2, scope, io))
                }
                (Op::LOr, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).bool();
                    if e1 {
                        return ControlFlow::Val(Val::Bool { val: true });
                    }
                    b!(self.interpret_expr(e2, scope, io))
                }
                (Op::Xor, [e1, e2]) => {
                    let e1 = b!(self.interpret_expr(e1, scope, io)).bool();
                    let e2 = b!(self.interpret_expr(e2, scope, io)).bool();
                    Val::Bool { val: e1 ^ e2 }
                }
                _ => unreachable!(),
            },
            Expr::Let { sym, bnd, bdy, .. } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                b!(scope.push(*sym, bnd, |scope| self.interpret_expr(bdy, scope, io)))
            }
            Expr::If { cnd, thn, els } => {
                if b!(self.interpret_expr(cnd, scope, io)).bool() {
                    b!(self.interpret_expr(thn, scope, io))
                } else {
                    b!(self.interpret_expr(els, scope, io))
                }
            }
            Expr::Apply { fun, args } => {
                let sym = b!(self.interpret_expr(fun, scope, io)).fun();

                let mut fn_args = Vec::new();
                for arg in args {
                    fn_args.push(b!(self.interpret_expr(arg, scope, io)));
                }

                self.interpret_fn(sym, fn_args, scope, io)
            }
            Expr::Loop { bdy } => loop {
                match self.interpret_expr(bdy, scope, io) {
                    ControlFlow::Return(val) => return ControlFlow::Return(val),
                    ControlFlow::Break(val) => return ControlFlow::Val(val),
                    ControlFlow::Continue | ControlFlow::Val(_) => {}
                }
            },
            Expr::Break { bdy } => {
                return ControlFlow::Break(b!(self.interpret_expr(bdy, scope, io)))
            }
            Expr::Seq { stmt, cnt } => {
                b!(self.interpret_expr(stmt, scope, io));
                b!(self.interpret_expr(cnt, scope, io))
            }
            Expr::Assign { sym, bnd } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                scope.0.insert(*sym, bnd);
                Val::Unit
            }
            Expr::Continue => return ControlFlow::Continue,
            Expr::Return { bdy } => {
                return ControlFlow::Return(b!(self.interpret_expr(bdy, scope, io)))
            }
            Expr::Struct { .. } => todo!(),
            Expr::Variant { .. } => todo!(),
            Expr::AccessField { .. } => todo!(),
            Expr::Switch { .. } => todo!(),
        })
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
        assert_eq!(io.outputs(), &expected_output);
    }

    test_each_file! { for ["test"] in "./programs/good" as interpreter => interpret }
}
