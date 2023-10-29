use crate::passes::atomize::{AExpr, Atom, PrgAtomized};
use crate::passes::parse::{Def, Lit};
use crate::passes::reveal_functions::{PrgRevealed, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};

impl<'p> PrgRevealed<'p> {
    pub fn atomize(self) -> PrgAtomized<'p> {
        PrgAtomized {
            defs: self
                .defs
                .into_iter()
                .map(|(sym, def)| {
                    let def = match def {
                        Def::Fn {
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
                .rfold(AExpr::Apply { fun, args }, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
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
        RExpr::Loop { bdy } => AExpr::Loop {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Break { bdy } => {
            let (atm, extras) = match bdy {
                Some(bdy) => atomize_atom(*bdy),
                None => {
                    return AExpr::Break {
                        bdy: Atom::Val { val: Lit::Unit },
                    }
                }
            };

            extras
                .into_iter()
                .rfold(AExpr::Break { bdy: atm }, |bdy, (sym, bnd)| AExpr::Let {
                    sym,
                    bnd: Box::new(bnd),
                    bdy: Box::new(bdy),
                })
        }
        RExpr::Seq { .. } => todo!(),
        RExpr::Assign { .. } => todo!(),
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
        | RExpr::Break { .. }
        | RExpr::Seq { .. }
        | RExpr::Assign { .. } => {
            let tmp = gen_sym("tmp");
            (Atom::Var { sym: tmp }, Some((tmp, atomize_expr(expr))))
        }
    }
}
