use crate::passes::atomize::{AExpr, Atom, PrgAtomized};
use crate::passes::parse::Def;
use crate::passes::reveal_functions::{PrgRevealed, RExpr};
use crate::utils::gen_sym::{gen_sym, UniqueSym};

impl<'p> PrgRevealed<'p> {
    #[must_use]
    pub fn atomize(self) -> PrgAtomized<'p> {
        todo!();
        // PrgAtomized {
        //     defs: self
        //         .defs
        //         .into_iter()
        //         .map(|(sym, def)| {
        //             let def = match def {
        //                 Def::Fn {
        //                     sym,
        //                     params,
        //                     typ,
        //                     bdy,
        //                 } => Def::Fn {
        //                     sym,
        //                     params,
        //                     typ,
        //                     bdy: atomize_expr(bdy),
        //                 },
        //                 Def::Struct { sym, fields } => Def::Struct { sym, fields },
        //                 Def::Enum { sym, variants } => Def::Enum { sym, variants },
        //             };
        //             (sym, def)
        //         })
        //         .collect(),
        //     entry: self.entry,
        // }
    }
}

fn atomize_expr(expr: RExpr) -> AExpr {
    match expr {
        RExpr::Lit { val, typ } => AExpr::Atom {
            atm: Atom::Val { val },
        },
        RExpr::Var { sym, typ } => AExpr::Atom {
            atm: Atom::Var { sym },
        },
        RExpr::Prim { op, args, typ } => {
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
        RExpr::Let { sym, bnd, bdy, typ } => AExpr::Let {
            sym,
            bnd: Box::new(atomize_expr(*bnd)),
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::If { cnd, thn, els, typ } => AExpr::If {
            cnd: Box::new(atomize_expr(*cnd)),
            thn: Box::new(atomize_expr(*thn)),
            els: Box::new(atomize_expr(*els)),
        },
        RExpr::Apply { fun, args, typ } => {
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
        RExpr::FunRef { sym, typ } => {
            let tmp = gen_sym("tmp");
            AExpr::Let {
                sym: tmp,
                bnd: Box::new(AExpr::FunRef { sym }),
                bdy: Box::new(AExpr::Atom {
                    atm: Atom::Var { sym: tmp },
                }),
            }
        }
        RExpr::Loop { bdy, typ } => AExpr::Loop {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Break { bdy, typ } => AExpr::Break {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Seq { stmt, cnt, typ } => AExpr::Seq {
            stmt: Box::new(atomize_expr(*stmt)),
            cnt: Box::new(atomize_expr(*cnt)),
        },
        RExpr::Assign { sym, bnd, typ } => AExpr::Assign {
            sym,
            bnd: Box::new(atomize_expr(*bnd)),
        },
        RExpr::Continue{typ} => AExpr::Continue,
        RExpr::Return { bdy, typ } => AExpr::Return {
            bdy: Box::new(atomize_expr(*bdy)),
        },
        RExpr::Struct { sym, fields, typ } => {
            let (fields, extras): (Vec<_>, Vec<_>) = fields
                .into_iter()
                .map(|(sym, expr)| {
                    let (field, extra) = atomize_atom(expr);
                    ((sym, field), extra)
                })
                .unzip();

            extras
                .into_iter()
                .flatten()
                .rfold(AExpr::Struct { sym, fields }, |bdy, (sym, bnd)| {
                    AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(bdy),
                    }
                })
        }
        RExpr::AccessField { strct, field, typ } => {
            let (strct, extra) = atomize_atom(*strct);

            extra
                .into_iter()
                .rfold(AExpr::AccessField { strct, field }, |bdy, (sym, bnd)| {
                    AExpr::Let {
                        sym,
                        bnd: Box::new(bnd),
                        bdy: Box::new(bdy),
                    }
                })
        }
    }
}

fn atomize_atom(expr: RExpr) -> (Atom, Option<(UniqueSym, AExpr)>) {
    if let RExpr::Lit { val, typ } = expr {
        (Atom::Val { val }, None)
    } else {
        let tmp = gen_sym("tmp");
        (Atom::Var { sym: tmp }, Some((tmp, atomize_expr(expr))))
    }
}
