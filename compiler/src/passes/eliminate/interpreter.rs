use crate::interpreter::Val;
use crate::interpreter::IO;
use crate::passes::atomize::Atom;
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use derive_more::Display;
use crate::passes::parse::{BinaryOp, UnaryOp};

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
    #[display(fmt = "stdlib function `{sym}`")]
    StdlibFunction { sym: &'p str },
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

impl<'p> From<EVal<'p>> for Val<'p> {
    fn from(value: EVal<'p>) -> Self {
        match value {
            EVal::Int { val } => Val::Int { val },
            EVal::Bool { val } => Val::Bool { val },
            EVal::Unit => Val::Unit,
            EVal::Function { sym } => Val::Function { sym },
            EVal::StdlibFunction { sym } => Val::StdlibFunction {sym},
        }
    }
}

impl<'p> From<TLit> for EVal<'p> {
    fn from(value: TLit) -> Self {
        match value {
            TLit::I64 { val } => Self::Int { val: val },
            TLit::U64 { val } => Self::Int { val: val as i64 },
            TLit::Bool { val } => Self::Bool { val },
            TLit::Unit => Self::Unit,
        }
    }
}

impl<'p> PrgEliminated<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> Vec<EVal<'p>> {
        let std_iter = self
            .std
            .iter()
            .map(|(_, &def)| (def, EVal::StdlibFunction { sym: def.sym }));

        self.interpret_tail(&self.blocks[&self.entry], &mut PushMap::from_iter(std_iter), io)
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
                .map(|atm| self.interpret_atom(atm, scope))
                .collect(),
            ETail::Seq { syms, bnd, tail } => {
                let bnds = syms
                    .iter()
                    .cloned()
                    .zip(self.interpret_expr(& bnd.inner, scope, io));
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
            EExpr::BinaryOp {
                op,
                exprs: [lhs, rhs],
            } => {
                let lhs = self.interpret_atom(&lhs, scope);
                let rhs = self.interpret_atom(&rhs, scope);
                match op {
                    BinaryOp::Add => EVal::Int {
                        val: lhs.int() + rhs.int(),
                    },
                    BinaryOp::Sub => EVal::Int {
                        val: lhs.int() - rhs.int(),
                    },
                    BinaryOp::Mul => EVal::Int {
                        val: lhs.int() * rhs.int(),
                    },
                    BinaryOp::Div => EVal::Int {
                        val: lhs.int() / rhs.int(),
                    },
                    BinaryOp::Mod => EVal::Int {
                        val: lhs.int() % rhs.int(),
                    },
                    BinaryOp::Xor => EVal::Bool {
                        val: lhs.bool() ^ rhs.bool(),
                    },
                    BinaryOp::GT => EVal::Bool {
                        val: lhs.int() > rhs.int(),
                    },
                    BinaryOp::GE => EVal::Bool {
                        val: lhs.int() >= rhs.int(),
                    },
                    BinaryOp::EQ => EVal::Bool {
                        val: lhs == rhs,
                    },
                    BinaryOp::LE => EVal::Bool {
                        val: lhs.int() <= rhs.int(),
                    },
                    BinaryOp::LT => EVal::Bool {
                        val: lhs.int() < rhs.int(),
                    },
                    BinaryOp::NE => EVal::Bool {
                        val: lhs != rhs,
                    },
                    BinaryOp::LAnd => EVal::Bool {
                        val: lhs.bool() && rhs.bool(),
                    },
                    BinaryOp::LOr => EVal::Bool {
                        val: lhs.bool() || rhs.bool(),
                    },
                }
            }
            EExpr::UnaryOp { op, expr } => {
                let expr = self.interpret_atom(&expr, scope);
                match op {
                    UnaryOp::Neg => EVal::Int { val: -expr.int() },
                    UnaryOp::Not => EVal::Bool { val: !expr.bool() },
                }
            }
            EExpr::Atom { atm, .. } => self.interpret_atom(atm, scope),
            EExpr::FunRef { sym, .. } => EVal::Function { sym: *sym },
            EExpr::Apply { fun, args, .. } => {
                let fun = self.interpret_atom(fun, scope);

                let mut fn_args = Vec::new();
                for atm in args {
                    fn_args.push(self.interpret_atom(atm, scope));
                }

                match fun {
                    EVal::StdlibFunction { sym } => {
                        match sym {
                            "exit" => {
                                unreachable!("Validated programs should not have an explicit call to exit yet.")
                            }
                            "print" => {
                                let val = fn_args[0].clone();
                                io.print(TLit::I64 { val: val.int() });
                                val
                            }
                            "read" => io.read().into(),
                            unknown => unreachable!(
                                "Encountered an undefined standard library function '{unknown}'"
                            ),
                        }
                    }
                    EVal::Function { sym } => {
                        let args = self.fn_params[&sym]
                            .iter()
                            .zip(fn_args.into_iter())
                            .map(|(param, val)| (param.sym, val));
                        return scope.push_iter(args, |scope| {
                            self.interpret_tail(&self.blocks[&sym], scope, io)
                        })
                    },
                    _ => unreachable!("The symbol did not refer to a function."),
                }
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
