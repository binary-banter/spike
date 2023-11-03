use crate::passes::eliminate_algebraic::PrgEliminated;
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;

// (Old variable name, field name) -> New variable name
type Ctx<'p> = HashMap<(UniqueSym<'p>, &'p str), UniqueSym<'p>>;

impl<'p> PrgExplicated<'p> {
    pub fn eliminate(self) -> PrgEliminated<'p> {
        let mut ctx = Ctx::new();
        self.blocks
            .values()
            .for_each(|tail| collect_tail(tail, &mut ctx));
        for (x, y) in &ctx {
            println!("old: {x:?} -> new: {y:?}");
        }

        todo!()
        // PrgEliminated {
        //     blocks: self.blocks.fmap(|tail| eliminate_tail(tail, &ctx)),
        //     fn_params: self.fn_params,
        //     entry: self.entry,
        // }
    }
}

fn collect_tail<'p>(tail: &Tail<'p, CExpr<'p>>, ctx: &mut Ctx<'p>) {
    match tail {
        //TODO what if a struct is returned from a function???
        Tail::Return { expr } => {}
        Tail::Seq { sym, bnd, tail } => {
            collect_expr(*sym, bnd, ctx);
            collect_tail(tail, ctx);
        }
        Tail::IfStmt { cnd, .. } => {}
        Tail::Goto { .. } => {}
    }
}

fn collect_expr<'p>(sym: UniqueSym<'p>, expr: &CExpr<'p>, ctx: &mut Ctx<'p>) {
    match expr {
        CExpr::Struct { fields, .. } => {
            for (field_sym, _) in fields {
                ctx.insert((sym, field_sym), gen_sym(sym.sym));
            }
        }
        _ => {}
    }
}

// fn eliminate_tail<'p>(tail: Tail<'p, CExpr<'p>>, ctx: &Ctx<'p>) -> Tail<'p, EExpr<'p>> {
//     match tail {
//         Tail::Return { expr } => Tail::Return {
//             expr: eliminate_expr(expr, ctx),
//         },
//         Tail::Seq {
//             sym,
//             bnd: CExpr::Struct { fields, .. },
//             tail,
//         } => fields
//             .into_iter()
//             .fold(eliminate_tail(*tail, ctx), |acc, (field_sym, field_bnd)| {
//                 Tail::Seq {
//                     sym: ctx[&(sym, field_sym)],
//                     bnd: EExpr::Atom { atm: field_bnd },
//                     tail: Box::new(acc),
//                 }
//             }),
//         Tail::Seq { sym, bnd, tail } => Tail::Seq {
//             sym,
//             bnd: eliminate_expr(bnd, ctx),
//             tail: Box::new(eliminate_tail(*tail, ctx)),
//         },
//         Tail::IfStmt { cnd, thn, els } => Tail::IfStmt {
//             cnd: eliminate_expr(cnd, ctx),
//             thn,
//             els,
//         },
//         Tail::Goto { lbl } => Tail::Goto { lbl },
//     }
// }
//
// fn eliminate_expr<'p>(expr: CExpr<'p>, ctx: &Ctx<'p>) -> EExpr<'p> {
//     match expr {
//         CExpr::Atom { atm } => EExpr::Atom { atm },
//         CExpr::Prim { op, args } => EExpr::Prim { op, args },
//         CExpr::Apply { fun, args } => EExpr::Apply { fun, args },
//         CExpr::FunRef { sym } => EExpr::FunRef { sym },
//         CExpr::Struct { .. } => {
//             unreachable!("Should've been handled in eliminate_tail")
//         }
//         CExpr::AccessField { strct, field } => EExpr::Atom {
//             atm: Atom::Var {
//                 sym: ctx[&(strct.var(), field)],
//             },
//         }
//     }
// }
