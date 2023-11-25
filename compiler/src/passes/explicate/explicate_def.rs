use std::collections::HashMap;
use crate::passes::atomize::DefAtomized;
use crate::passes::explicate::explicate;
use crate::passes::explicate::explicate::Env;
use crate::passes::explicate::explicate_tail::explicate_tail;
use crate::passes::parse::{Def, TypeDef};
use crate::utils::gen_sym::UniqueSym;

pub fn explicate_def<'p>(
    def: DefAtomized<'p>,
    env: &mut Env<'_, 'p>,
    defs: &mut HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) {
    match def {
        Def::Fn { sym, bdy, .. } => {
            let tail = explicate_tail(bdy, env);
            env.blocks.insert(sym, tail);
        }
        Def::TypeDef { sym, def } => {
            defs.insert(sym, def);
        }
    }
}
