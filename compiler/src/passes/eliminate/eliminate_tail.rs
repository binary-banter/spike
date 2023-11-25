use crate::passes::atomize::Atom;
use crate::passes::eliminate::eliminate::Ctx;
use crate::passes::eliminate::eliminate_expr::eliminate_expr;
use crate::passes::eliminate::eliminate_params::flatten_type;
use crate::passes::eliminate::eliminate_seq::eliminate_seq;
use crate::passes::eliminate::ETail;
use crate::passes::explicate::CTail;
use crate::passes::parse::TypeDef;
use crate::utils::gen_sym::UniqueSym;
use functor_derive::Functor;
use std::collections::HashMap;

pub fn eliminate_tail<'p>(
    tail: CTail<'p>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    match tail {
        CTail::Return { expr } => match expr.inner {
            Atom::Val { val } => ETail::Return {
                exprs: vec![(Atom::Val { val })],
            },
            Atom::Var { sym } => ETail::Return {
                exprs: flatten_type(sym, &expr.meta, ctx, defs)
                    .fmap(|(sym, _)| (Atom::Var { sym })),
            },
        },
        CTail::Seq { sym, bnd, tail } => {
            let tail = eliminate_tail(*tail, ctx, defs);
            eliminate_seq(sym, ctx, bnd, tail, defs)
        }
        CTail::IfStmt { cnd, thn, els } => ETail::IfStmt {
            cnd: eliminate_expr(cnd),
            thn,
            els,
        },
        CTail::Goto { lbl } => ETail::Goto { lbl },
    }
}
