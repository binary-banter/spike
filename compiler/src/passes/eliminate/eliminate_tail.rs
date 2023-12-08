use crate::passes::atomize::Atom;
use crate::passes::eliminate::eliminate::Ctx;
use crate::passes::eliminate::eliminate_expr::eliminate_expr;
use crate::passes::eliminate::eliminate_params::flatten_type;
use crate::passes::eliminate::eliminate_seq::eliminate_seq;
use crate::passes::eliminate::TailEliminated;
use crate::passes::explicate::TailExplicated;
use crate::passes::parse::TypeDef;
use crate::utils::unique_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

pub fn eliminate_tail<'p>(
    tail: TailExplicated<'p>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> TailEliminated<'p> {
    match tail {
        TailExplicated::Return { expr } => match expr.inner {
            Atom::Val { val } => TailEliminated::Return {
                exprs: vec![(Atom::Val { val })],
            },
            Atom::Var { sym } => TailEliminated::Return {
                exprs: flatten_type(sym, &expr.meta, ctx, defs)
                    .fmap(|(sym, _)| (Atom::Var { sym })),
            },
        },
        TailExplicated::Seq { sym, bnd, tail } => {
            let tail = eliminate_tail(*tail, ctx, defs);
            eliminate_seq(sym, ctx, bnd, tail, defs)
        }
        TailExplicated::IfStmt { cnd, thn, els } => TailEliminated::IfStmt {
            cnd: eliminate_expr(cnd),
            thn,
            els,
        },
        TailExplicated::Goto { lbl } => TailEliminated::Goto { lbl },
    }
}
