use crate::interpreter::Val;
use crate::interpreter::IO;
use crate::passes::atomize::Atom;
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use crate::passes::parse::{BinaryOp, UnaryOp};

impl<'p> PrgEliminated<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> Vec<Val<'p>> {
        let std_iter = self
            .std
            .iter()
            .map(|(_, &def)| (def, Val::StdlibFunction { sym: def.sym }));

        self.interpret_tail(&self.blocks[&self.entry], &mut PushMap::from_iter(std_iter), io)
    }

    fn interpret_tail(
        &self,
        tail: &ETail<'p>,
        scope: &mut PushMap<UniqueSym<'p>, Val<'p>>,
        io: &mut impl IO,
    ) -> Vec<Val<'p>> {
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
        scope: &mut PushMap<UniqueSym<'p>, Val<'p>>,
        io: &mut impl IO,
    ) -> Vec<Val<'p>> {
        let val = match expr {
            EExpr::BinaryOp {
                op,
                exprs: [lhs, rhs],
            } => {
                let lhs = self.interpret_atom(&lhs, scope);
                let rhs = self.interpret_atom(&rhs, scope);
                match op {
                    BinaryOp::Add => Val::Int {
                        val: lhs.int() + rhs.int(),
                    },
                    BinaryOp::Sub => Val::Int {
                        val: lhs.int() - rhs.int(),
                    },
                    BinaryOp::Mul => Val::Int {
                        val: lhs.int() * rhs.int(),
                    },
                    BinaryOp::Div => Val::Int {
                        val: lhs.int() / rhs.int(),
                    },
                    BinaryOp::Mod => Val::Int {
                        val: lhs.int() % rhs.int(),
                    },
                    BinaryOp::Xor => Val::Bool {
                        val: lhs.bool() ^ rhs.bool(),
                    },
                    BinaryOp::GT => Val::Bool {
                        val: lhs.int() > rhs.int(),
                    },
                    BinaryOp::GE => Val::Bool {
                        val: lhs.int() >= rhs.int(),
                    },
                    BinaryOp::EQ => Val::Bool {
                        val: lhs == rhs,
                    },
                    BinaryOp::LE => Val::Bool {
                        val: lhs.int() <= rhs.int(),
                    },
                    BinaryOp::LT => Val::Bool {
                        val: lhs.int() < rhs.int(),
                    },
                    BinaryOp::NE => Val::Bool {
                        val: lhs != rhs,
                    },
                    BinaryOp::LAnd => Val::Bool {
                        val: lhs.bool() && rhs.bool(),
                    },
                    BinaryOp::LOr => Val::Bool {
                        val: lhs.bool() || rhs.bool(),
                    },
                }
            }
            EExpr::UnaryOp { op, expr } => {
                let expr = self.interpret_atom(&expr, scope);
                match op {
                    UnaryOp::Neg => Val::Int { val: -expr.int() },
                    UnaryOp::Not => Val::Bool { val: !expr.bool() },
                }
            }
            EExpr::Atom { atm, .. } => self.interpret_atom(atm, scope),
            EExpr::FunRef { sym, .. } => Val::Function { sym: *sym },
            EExpr::Apply { fun, args, .. } => {
                let fun = self.interpret_atom(fun, scope);

                let mut fn_args = Vec::new();
                for atm in args {
                    fn_args.push(self.interpret_atom(atm, scope));
                }

                match fun {
                    Val::StdlibFunction { sym } => {
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
                    Val::Function { sym } => {
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
        scope: &PushMap<UniqueSym<'p>, Val<'p>>,
    ) -> Val<'p> {
        match atom {
            Atom::Val { val } => (*val).into(),
            Atom::Var { sym } => scope[sym].clone(),
        }
    }
}
