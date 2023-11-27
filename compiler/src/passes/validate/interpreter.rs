use crate::interpreter::{Val, IO};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Meta, UnaryOp};
use crate::passes::validate::{ExprValidated, PrgValidated, TLit};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;

#[derive(Clone)]
pub enum ControlFlow<'p> {
    Val(Val<'p>),
    Break(Val<'p>),
    Return(Val<'p>),
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

impl<'p> PrgValidated<'p> {
    pub fn interpret(&'p self, io: &mut impl IO) -> Val<'p> {
        let std_iter = self
            .std
            .iter()
            .map(|(_, &def)| (def, Val::StdlibFunction { sym: def.sym }));

        // Create a scope with all global definitions.
        let mut scope = PushMap::from_iter(
            self.defs
                .iter()
                .map(|(&sym, _)| (sym, Val::Function { sym }))
                // Include the standard library in the scope.
                .chain(std_iter),
        );

        // Interpret the program starting from the entry point.
        self.interpret_fn(self.entry, Vec::new(), &mut scope, io)
    }

    fn interpret_fn(
        &'p self,
        sym: UniqueSym<'p>,
        args: Vec<Val<'p>>,
        scope: &mut PushMap<UniqueSym<'p>, Val<'p>>,
        io: &mut impl IO,
    ) -> Val<'p> {
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
        expr: &'p Meta<Type<UniqueSym>, ExprValidated<'p>>,
        scope: &mut PushMap<UniqueSym<'p>, Val<'p>>,
        io: &mut impl IO,
    ) -> ControlFlow<'p> {
        ControlFlow::Val(match &expr.inner {
            ExprValidated::Lit { val, .. } => (*val).into(),
            ExprValidated::Var { sym, .. } => scope[sym].clone(),
            ExprValidated::BinaryOp {
                op,
                exprs: [lhs, rhs],
            } => {
                let lhs = b!(self.interpret_expr(lhs, scope, io));
                let mut rhs = || self.interpret_expr(rhs, scope, io);
                match op {
                    BinaryOp::Add => Val::Int {
                        val: lhs.int() + b!(rhs()).int(),
                    },
                    BinaryOp::Sub => Val::Int {
                        val: lhs.int() - b!(rhs()).int(),
                    },
                    BinaryOp::Mul => Val::Int {
                        val: lhs.int() * b!(rhs()).int(),
                    },
                    BinaryOp::Div => Val::Int {
                        val: lhs.int() / b!(rhs()).int(),
                    },
                    BinaryOp::Mod => Val::Int {
                        val: lhs.int() % b!(rhs()).int(),
                    },
                    BinaryOp::Xor => Val::Bool {
                        val: lhs.bool() ^ b!(rhs()).bool(),
                    },
                    BinaryOp::GT => Val::Bool {
                        val: lhs.int() > b!(rhs()).int(),
                    },
                    BinaryOp::GE => Val::Bool {
                        val: lhs.int() >= b!(rhs()).int(),
                    },
                    BinaryOp::EQ => Val::Bool {
                        val: lhs == b!(rhs()),
                    },
                    BinaryOp::LE => Val::Bool {
                        val: lhs.int() <= b!(rhs()).int(),
                    },
                    BinaryOp::LT => Val::Bool {
                        val: lhs.int() < b!(rhs()).int(),
                    },
                    BinaryOp::NE => Val::Bool {
                        val: lhs != b!(rhs()),
                    },
                    BinaryOp::LAnd => {
                        // Short-circuit logical AND.
                        if !lhs.bool() {
                            return ControlFlow::Val(lhs);
                        }
                        b!(rhs())
                    }
                    BinaryOp::LOr => {
                        // Short-circuit logical OR.
                        if lhs.bool() {
                            return ControlFlow::Val(lhs);
                        }
                        b!(rhs())
                    }
                }
            }
            ExprValidated::UnaryOp { op, expr } => {
                let expr = b!(self.interpret_expr(expr, scope, io));
                match op {
                    UnaryOp::Neg => Val::Int { val: -expr.int() },
                    UnaryOp::Not => Val::Bool { val: !expr.bool() },
                }
            }
            ExprValidated::Let { sym, bnd, bdy, .. } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                b!(scope.push(*sym, bnd, |scope| self.interpret_expr(bdy, scope, io)))
            }
            ExprValidated::If { cnd, thn, els, .. } => {
                if b!(self.interpret_expr(cnd, scope, io)).bool() {
                    b!(self.interpret_expr(thn, scope, io))
                } else {
                    b!(self.interpret_expr(els, scope, io))
                }
            }
            ExprValidated::Apply { fun, args, .. } => {
                let fun = b!(self.interpret_expr(fun, scope, io));

                let mut fn_args = Vec::new();
                for arg in args {
                    fn_args.push(b!(self.interpret_expr(arg, scope, io)));
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
                    Val::Function { sym } => self.interpret_fn(sym, fn_args, scope, io),
                    _ => unreachable!("The symbol did not refer to a function."),
                }
            }
            ExprValidated::Loop { bdy, .. } => loop {
                match self.interpret_expr(bdy, scope, io) {
                    ControlFlow::Return(val) => return ControlFlow::Return(val),
                    ControlFlow::Break(val) => return ControlFlow::Val(val),
                    ControlFlow::Continue | ControlFlow::Val(_) => {}
                }
            },
            ExprValidated::Break { bdy, .. } => {
                return ControlFlow::Break(b!(self.interpret_expr(bdy, scope, io)))
            }
            ExprValidated::Seq { stmt, cnt, .. } => {
                b!(self.interpret_expr(stmt, scope, io));
                b!(self.interpret_expr(cnt, scope, io))
            }
            ExprValidated::Assign { sym, bnd, .. } => {
                let bnd = b!(self.interpret_expr(bnd, scope, io));
                scope.0.insert(*sym, bnd);
                Val::Unit
            }
            ExprValidated::Continue { .. } => return ControlFlow::Continue,
            ExprValidated::Return { bdy, .. } => {
                return ControlFlow::Return(b!(self.interpret_expr(bdy, scope, io)))
            }
            ExprValidated::Struct { fields, .. } => {
                let mut field_values = HashMap::new();
                for (sym, field) in fields {
                    field_values.insert(*sym, b!(self.interpret_expr(field, scope, io)));
                }
                Val::StructInstance {
                    fields: field_values,
                }
            }
            ExprValidated::AccessField { strct, field, .. } => {
                let s = b!(self.interpret_expr(strct, scope, io));
                s.strct()[field].clone()
            }
            ExprValidated::Variant { .. } => todo!(),
            ExprValidated::Switch { .. } => todo!(),
            ExprValidated::Asm { .. } => todo!(),
        })
    }
}
