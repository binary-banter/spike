//! This pass compiles `ULVarProgram`s  into `ALVarProgram` in which the arguments of operations are atomic expressions.
//!
//! This is accomplished by introducing new temporary variables, assigning
//! the complex operand to those new variables, and then using them in place
//! of the complex operand.
//!
//! We consider `Int`s and `Var`s atomic.

use crate::language::alvar::{ADef, AExpr, Atom, PrgAtomized};
use crate::language::rlvar::{PrgRevealed, RDef, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};

impl<'p> PrgRevealed<'p> {
    /// See module-level documentation.
    pub fn atomize(self) -> PrgAtomized<'p> {
        PrgAtomized {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| {
                    let def = match def {
                        RDef::Fn {
                            sym,
                            params,
                            typ,
                            bdy,
                        } => ADef::Fn {
                            sym,
                            params,
                            typ,
                            bdy: rco_expr(bdy),
                        },
                    };
                    (sym, def)
                })
                .collect(),
            entry: self.entry,
        }
    }
}

fn rco_expr(expr: RExpr) -> AExpr {
    match expr {
        RExpr::Lit { val } => AExpr::Atom {
            atm: Atom::Val { val },
        },
        RExpr::Var { sym } => AExpr::Atom {
            atm: Atom::Var { sym },
        },
        RExpr::Prim { op, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(rco_atom).unzip();

            extras
                .into_iter()
                .flatten()
                .rfold(AExpr::Prim { op, args }, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
        }
        RExpr::Let { sym, bnd, bdy } => AExpr::Let {
            sym,
            bnd: Box::new(rco_expr(*bnd)),
            bdy: Box::new(rco_expr(*bdy)),
        },
        RExpr::If { cnd, thn, els } => AExpr::If {
            cnd: Box::new(rco_expr(*cnd)),
            thn: Box::new(rco_expr(*thn)),
            els: Box::new(rco_expr(*els)),
        },
        RExpr::Apply { fun, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(rco_atom).unzip();

            let (fun, fun_expr) = rco_atom(*fun);

            fun_expr
                .into_iter()
                .chain(extras.into_iter().flatten())
                .rfold(
                    AExpr::Apply {
                        fun: Box::new(fun),
                        args,
                    },
                    |bdy, (sym, bnd)| AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(bdy),
                    },
                )
        }
        RExpr::FunRef { sym } => {
            let tmp = gen_sym("tmp");
            AExpr::Let {
                sym: tmp,
                bnd: Box::new(AExpr::FunRef { sym }),
                bdy: Box::new(AExpr::Atom {
                    atm: Atom::Var { sym: tmp },
                }),
            }
        }
    }
}

fn rco_atom(expr: RExpr) -> (Atom, Option<(UniqueSym, AExpr)>) {
    match expr {
        RExpr::Lit { val } => (Atom::Val { val }, None),
        RExpr::Var { sym } => (Atom::Var { sym }, None),
        RExpr::Prim { .. }
        | RExpr::Let { .. }
        | RExpr::If { .. }
        | RExpr::Apply { .. }
        | RExpr::FunRef { .. } => {
            let tmp = gen_sym("tmp");
            (Atom::Var { sym: tmp }, Some((tmp, rco_expr(expr))))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::passes::parse::PrgGenericVar;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn atomize([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: PrgGenericVar<_> = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .into();
        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as atomize => atomize }
}
