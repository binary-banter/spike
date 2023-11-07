use crate::interpreter::Val;
use crate::interpreter::IO;
use crate::passes::atomize::Atom;
use crate::passes::eliminate_algebraic::{EExpr, ETail, PrgEliminated};
use crate::passes::parse::Op;
use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use derive_more::Display;

#[derive(Eq, PartialEq, Copy, Clone, Debug, Display)]
pub enum EVal<'p> {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "fn pointer `{sym}`")]
    Function { sym: UniqueSym<'p> },
}

impl<'p> EVal<'p> {
    pub fn int(&self) -> i64 {
        match self {
            Self::Int { val } => *val,
            _ => panic!(),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            Self::Bool { val } => *val,
            _ => panic!(),
        }
    }

    pub fn fun(&self) -> UniqueSym<'p> {
        match self {
            Self::Function { sym } => *sym,
            _ => panic!(),
        }
    }
}

impl<'p> From<EVal<'p>> for Val<'p, UniqueSym<'p>> {
    fn from(value: EVal<'p>) -> Self {
        match value {
            EVal::Int { val } => Val::Int { val },
            EVal::Bool { val } => Val::Bool { val },
            EVal::Unit => Val::Unit,
            EVal::Function { sym } => Val::Function { sym },
        }
    }
}

impl<'p> From<TLit> for EVal<'p> {
    fn from(value: TLit) -> Self {
        match value {
            TLit::Int { val } => Self::Int { val: val as i64 },
            TLit::Bool { val } => Self::Bool { val },
            TLit::Unit => Self::Unit,
        }
    }
}

impl<'p> PrgEliminated<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> Vec<EVal<'p>> {
        self.interpret_tail(&self.blocks[&self.entry], &mut PushMap::default(), io)
    }

    fn interpret_tail(
        &self,
        tail: &ETail<'p>,
        scope: &mut PushMap<UniqueSym<'p>, EVal<'p>>,
        io: &mut impl IO,
    ) -> Vec<EVal<'p>> {
        match tail {
            ETail::Return { exprs: expr } => expr
                .iter()
                .map(|(atm, _)| self.interpret_atom(atm, scope))
                .collect(),
            ETail::Seq { syms, bnd, tail } => {
                let bnds = syms
                    .iter()
                    .cloned()
                    .zip(self.interpret_expr(bnd, scope, io));
                scope.push_iter(bnds, |scope| self.interpret_tail(tail, scope, io))
            }
            ETail::IfStmt { cnd, thn, els } => {
                if self.interpret_expr(cnd, scope, io)[0].bool() {
                    self.interpret_tail(&self.blocks[thn], scope, io)
                } else {
                    self.interpret_tail(&self.blocks[els], scope, io)
                }
            }
            ETail::Goto { lbl } => self.interpret_tail(&self.blocks[lbl], scope, io),
        }
    }

    pub fn interpret_expr(
        &self,
        expr: &EExpr<'p>,
        scope: &mut PushMap<UniqueSym<'p>, EVal<'p>>,
        io: &mut impl IO,
    ) -> Vec<EVal<'p>> {
        let val = match expr {
            EExpr::Prim { op, args, .. } => match (op, args.as_slice()) {
                (Op::Read, []) => io.read().into(),
                (Op::Print, [v]) => {
                    let val = self.interpret_atom(v, scope);
                    io.print(TLit::Int {
                        val: val.int() as i32,
                    });
                    val
                }
                (Op::Plus, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Int { val: e1 + e2 }
                }
                (Op::Minus, [e1]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    EVal::Int { val: -e1 }
                }
                (Op::Minus, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Int { val: e1 - e2 }
                }
                (Op::Mul, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Int { val: e1 * e2 }
                }
                (Op::Div, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Int { val: e1 / e2 }
                }
                (Op::Mod, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Int { val: e1 % e2 }
                }
                (Op::GT, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Bool { val: e1 > e2 }
                }
                (Op::GE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Bool { val: e1 >= e2 }
                }
                (Op::LT, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Bool { val: e1 < e2 }
                }
                (Op::LE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).int();
                    let e2 = self.interpret_atom(e2, scope).int();
                    EVal::Bool { val: e1 <= e2 }
                }
                (Op::EQ, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope);
                    let e2 = self.interpret_atom(e2, scope);
                    EVal::Bool { val: e1 == e2 }
                }
                (Op::NE, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope);
                    let e2 = self.interpret_atom(e2, scope);
                    EVal::Bool { val: e1 != e2 }
                }
                (Op::Not, [e1]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    EVal::Bool { val: !e1 }
                }
                (Op::LAnd, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    if !e1 {
                        EVal::Bool { val: false }
                    } else {
                        self.interpret_atom(e2, scope)
                    }
                }
                (Op::LOr, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    if e1 {
                        EVal::Bool { val: true }
                    } else {
                        self.interpret_atom(e2, scope)
                    }
                }
                (Op::Xor, [e1, e2]) => {
                    let e1 = self.interpret_atom(e1, scope).bool();
                    let e2 = self.interpret_atom(e2, scope).bool();
                    EVal::Bool { val: e1 ^ e2 }
                }
                _ => unreachable!(),
            },
            EExpr::Atom { atm, .. } => self.interpret_atom(atm, scope),
            EExpr::FunRef { sym, .. } => EVal::Function { sym: *sym },
            EExpr::Apply { fun, args, .. } => {
                let fn_sym = self.interpret_atom(fun, scope).fun();
                let args = self.fn_params[&fn_sym]
                    .iter()
                    .zip(args.iter())
                    .map(|(param, (atm, _))| (param.sym, self.interpret_atom(atm, scope)))
                    .collect::<Vec<_>>();

                return scope.push_iter(args.into_iter(), |scope| {
                    self.interpret_tail(&self.blocks[&fn_sym], scope, io)
                });
            }
        };
        vec![val]
    }

    #[must_use]
    pub fn interpret_atom(
        &self,
        atom: &Atom<'p>,
        scope: &PushMap<UniqueSym<'p>, EVal<'p>>,
    ) -> EVal<'p> {
        match atom {
            Atom::Val { val } => (*val).into(),
            Atom::Var { sym } => scope[sym],
        }
    }
}
