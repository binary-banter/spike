use crate::interpreter::value::Val;
use crate::interpreter::IO;
use crate::language::cvar::{CExpr, PrgExplicated, Tail};
use crate::passes::atomize::Atom;
use crate::passes::parse::{Lit, Op};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

impl<'p> PrgExplicated<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> Val<UniqueSym<'p>> {
        self.interpret_tail(&self.blocks[&self.entry], &mut PushMap::default(), io)
    }

    fn interpret_tail(
        &self,
        tail: &Tail<'p>,
        scope: &mut PushMap<UniqueSym<'p>, Val<UniqueSym<'p>>>,
        io: &mut impl IO,
    ) -> Val<UniqueSym<'p>> {
        match tail {
            Tail::Return { expr } => self.interpret_expr(&expr.clone(), scope, io),
            Tail::Seq { sym, bnd, tail } => {
                let bnd = self.interpret_expr(&bnd.clone(), scope, io);
                scope.push(*sym, bnd, |scope| self.interpret_tail(tail, scope, io))
            }
            Tail::IfStmt { cnd, thn, els } => {
                if self.interpret_expr(&cnd.clone(), scope, io).bool() {
                    self.interpret_tail(&self.blocks[thn], scope, io)
                } else {
                    self.interpret_tail(&self.blocks[els], scope, io)
                }
            }
            Tail::Goto { lbl } => self.interpret_tail(&self.blocks[lbl], scope, io),
        }
    }

    pub fn interpret_atom(
        &self,
        atom: &Atom<'p>,
        scope: &PushMap<UniqueSym<'p>, Val<UniqueSym<'p>>>,
    ) -> Val<UniqueSym<'p>> {
        match atom {
            Atom::Val { val } => (*val).into(),
            Atom::Var { sym } => scope[sym],
        }
    }

    pub fn interpret_expr(
        &self,
        expr: &CExpr<'p>,
        scope: &mut PushMap<UniqueSym<'p>, Val<UniqueSym<'p>>>,
        io: &mut impl IO,
    ) -> Val<UniqueSym<'p>> {
        match expr {
            CExpr::Prim { op, args } => match (op, args.as_slice()) {
                (Op::Read, []) => io.read().into(),
                (Op::Print, [v]) => {
                    let val = self.interpret_atom(v, scope);
                    io.print(Lit::Int { val: val.int() });
                    val
                }
                (Op::Plus, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Int { val: e1 + e2 }
                }
                (Op::Minus, [e1]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    Val::Int { val: -e1 }
                }
                (Op::Minus, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Int { val: e1 - e2 }
                }
                (Op::Mul, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Int { val: e1 * e2 }
                }
                (Op::Div, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Int { val: e1 / e2 }
                }
                (Op::Mod, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Int { val: e1 % e2 }
                }
                (Op::GT, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Bool { val: e1 > e2 }
                }
                (Op::GE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Bool { val: e1 >= e2 }
                }
                (Op::LT, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Bool { val: e1 < e2 }
                }
                (Op::LE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    Val::Bool { val: e1 <= e2 }
                }
                (Op::EQ, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope);
                    let e2 = self.interpret_atom(e2, scope);
                    Val::Bool { val: e1 == e2 }
                }
                (Op::NE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope);
                    let e2 = self.interpret_atom(e2, scope);
                    Val::Bool { val: e1 != e2 }
                }
                (Op::Not, [e1]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    Val::Bool { val: !e1 }
                }
                (Op::LAnd, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    if !e1 {
                        return Val::Bool { val: false };
                    }
                    self.interpret_atom(e2, scope)
                }
                (Op::LOr, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    if e1 {
                        return Val::Bool { val: true };
                    }
                    self.interpret_atom(e2, scope)
                }
                (Op::Xor, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    let e2 = self.interpret_atom(e2, scope).bool();
                    Val::Bool { val: e1 ^ e2 }
                }
                _ => unreachable!(),
            },
            CExpr::Atom { atm } => self.interpret_atom(atm, scope),
            CExpr::FunRef { sym } => Val::Function { sym: *sym },
            CExpr::Apply { fun, args } => {
                let fn_sym = self.interpret_atom(fun, scope).fun();
                let args = self.fn_params[&fn_sym]
                    .iter()
                    .zip(args.iter())
                    .map(|(sym, atm)| (*sym, self.interpret_atom(atm, scope)))
                    .collect::<Vec<_>>();

                scope.push_iter(args.into_iter(), |scope| {
                    self.interpret_tail(&self.blocks[&fn_sym], scope, io)
                })
            }
        }
    }
}
