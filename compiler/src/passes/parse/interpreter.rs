use crate::interpreter::Val;
use crate::interpreter::IO;
use crate::passes::parse::{Def, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::passes::validate::{TExpr, TLit};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

#[derive(Clone)]
pub enum ControlFlow<'p, A: Copy + Hash + Eq + Display> {
    Val(Val<'p, A>),
    Break(Val<'p, A>),
    Return(Val<'p, A>),
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

impl<'p> PrgUniquified<'p> {
    pub fn interpret(&'p self, io: &mut impl IO) -> Val<'p, UniqueSym<'p>> {
        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|(&sym, _)| (sym, Val::Function { sym })),
        );
        self.interpret_fn(self.entry, Vec::new(), &mut scope, io)
    }

    fn interpret_fn(
        &'p self,
        sym: UniqueSym<'p>,
        args: Vec<Val<'p, UniqueSym<'p>>>,
        scope: &mut PushMap<UniqueSym<'p>, Val<'p, UniqueSym<'p>>>,
        io: &mut impl IO,
    ) -> Val<'p, UniqueSym<'p>> {
        match &self.defs[&sym] {
            Def::Fn { params, bdy, .. } => scope.push_iter(
                params
                    .iter()
                    .zip(args.iter())
                    .map(|(param, v)| (param.sym, v.clone())),
                |scope| match self.interpret_expr(bdy, scope, io) {
                    ControlFlow::Return(val) | ControlFlow::Val(val) => val,
                    ControlFlow::Continue | ControlFlow::Break(_) => unreachable!(),
                },
            ),
            Def::TypeDef { .. } => unreachable!(),
        }
    }

    pub fn interpret_expr(
        &'p self,
        expr: &'p TExpr<UniqueSym<'p>>,
        scope: &mut PushMap<UniqueSym<'p>, Val<'p, UniqueSym<'p>>>,
        io: &mut impl IO,
    ) -> ControlFlow<'p, UniqueSym<'p>> {
        ControlFlow::Val(match expr {
            TExpr::Lit { val, .. } => (*val).into(),
            TExpr::Var { sym, .. } => scope[sym].clone(),
            TExpr::Prim { op, args, .. } => match (op, args.as_slice()) {
                (Op::Read, []) => io.read().into(),
                (Op::Print, [v]) => {
                    let val = b!(self.interpret_expr(v, scope, io));
                    io.print(TLit::Int {
                        val: val.int() as i32,
                    });
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
            TExpr::Let { sym, bnd, bdy, .. } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                b!(scope.push(*sym, bnd, |scope| self.interpret_expr(bdy, scope, io)))
            }
            TExpr::If { cnd, thn, els, .. } => {
                if b!(self.interpret_expr(cnd, scope, io)).bool() {
                    b!(self.interpret_expr(thn, scope, io))
                } else {
                    b!(self.interpret_expr(els, scope, io))
                }
            }
            TExpr::Apply { fun, args, .. } => {
                let sym = b!(self.interpret_expr(fun, scope, io)).fun();

                let mut fn_args = Vec::new();
                for arg in args {
                    fn_args.push(b!(self.interpret_expr(arg, scope, io)));
                }

                self.interpret_fn(sym, fn_args, scope, io)
            }
            TExpr::Loop { bdy, .. } => loop {
                match self.interpret_expr(bdy, scope, io) {
                    ControlFlow::Return(val) => return ControlFlow::Return(val),
                    ControlFlow::Break(val) => return ControlFlow::Val(val),
                    ControlFlow::Continue | ControlFlow::Val(_) => {}
                }
            },
            TExpr::Break { bdy, .. } => {
                return ControlFlow::Break(b!(self.interpret_expr(bdy, scope, io)))
            }
            TExpr::Seq { stmt, cnt, .. } => {
                b!(self.interpret_expr(stmt, scope, io));
                b!(self.interpret_expr(cnt, scope, io))
            }
            TExpr::Assign { sym, bnd, .. } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                scope.0.insert(*sym, bnd);
                Val::Unit
            }
            TExpr::Continue { .. } => return ControlFlow::Continue,
            TExpr::Return { bdy, .. } => {
                return ControlFlow::Return(b!(self.interpret_expr(bdy, scope, io)))
            }
            TExpr::Struct { fields, .. } => {
                let mut field_values = HashMap::new();
                for (sym, field) in fields {
                    field_values.insert(*sym, b!(self.interpret_expr(field, scope, io)));
                }
                Val::StructInstance {
                    fields: field_values,
                }
            }
            TExpr::Variant { .. } => todo!(),
            TExpr::AccessField { strct, field, .. } => {
                let s = b!(self.interpret_expr(strct, scope, io));
                s.strct()[field].clone()
            }
            TExpr::Switch { .. } => todo!(),
        })
    }
}
