use crate::passes::atomize::{AExpr, Atom, PrgAtomized};
use crate::passes::parse::{Def, Lit};
use crate::passes::reveal_functions::{PrgRevealed, RDef, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};

impl<'p> PrgRevealed<'p> {
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
                        } => Def::Fn {
                            sym,
                            params,
                            typ,
                            bdy: atomize_expr(bdy),
                        },
                    };
                    (sym, def)
                })
                .collect(),
            entry: self.entry,
        }
    }
}

fn atomize_expr(expr: RExpr) -> AExpr {
    match expr {
        RExpr::Lit { val } => AExpr::Atom {
            atm: Atom::Val { val },
        },
        RExpr::Var { sym } => AExpr::Atom {
            atm: Atom::Var { sym },
        },
        RExpr::Prim { op, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(atomize_atom).unzip();

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
            bnd: Box::new(atomize_expr(*bnd)),
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::If { cnd, thn, els } => AExpr::If {
            cnd: Box::new(atomize_expr(*cnd)),
            thn: Box::new(atomize_expr(*thn)),
            els: Box::new(atomize_expr(*els)),
        },
        RExpr::Apply { fun, args } => {
            let (args, extras): (Vec<_>, Vec<_>) = args.into_iter().map(atomize_atom).unzip();

            let (fun, fun_expr) = atomize_atom(*fun);

            fun_expr
                .into_iter()
                .chain(extras.into_iter().flatten())
                .rfold(
                    AExpr::Apply {
                        fun,
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
        RExpr::Loop { bdy } => AExpr::Loop { bdy: Box::new(atomize_expr(*bdy))},
        RExpr::Break { bdy } => {
            let (atm, extras) = match bdy {
                Some(bdy) => atomize_atom(*bdy),
                None => return AExpr::Break { bdy: Atom::Val { val: Lit::Unit }},
            };

            extras
                .into_iter()
                .rfold(AExpr::Break { bdy: atm}, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
        },
    }
}

fn atomize_atom(expr: RExpr) -> (Atom, Option<(UniqueSym, AExpr)>) {
    match expr {
        RExpr::Lit { val } => (Atom::Val { val }, None),
        RExpr::Var { sym } => (Atom::Var { sym }, None),
        RExpr::Prim { .. }
        | RExpr::Let { .. }
        | RExpr::If { .. }
        | RExpr::Apply { .. }
        | RExpr::FunRef { .. }
        | RExpr::Loop { .. }
        |  RExpr::Break { .. } => {
            let tmp = gen_sym("tmp");
            (Atom::Var { sym: tmp }, Some((tmp, atomize_expr(expr))))
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