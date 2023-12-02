use crate::passes::atomize::{DefAtomized, PrgAtomized};
use crate::passes::explicate::{TailExplicated, PrgExplicated, FunExplicated};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::explicate::explicate_tail::explicate_tail;

pub struct Env<'a, 'p> {
    pub blocks: &'a mut HashMap<UniqueSym<'p>, TailExplicated<'p>>,
    /// (block to jump to, variable to write to)
    pub break_target: Option<(UniqueSym<'p>, UniqueSym<'p>)>,
    /// block to jump to
    pub continue_target: Option<UniqueSym<'p>>,
}

impl<'p> PrgAtomized<'p> {
    #[must_use]
    pub fn explicate(self) -> PrgExplicated<'p> {
        let mut fns = HashMap::new();
        let mut defs = HashMap::new();

        for (entry, def) in self.defs {
            match def {
                DefAtomized::Fn { sym, params, bdy, .. } => {
                    let mut blocks = HashMap::new();
                    let mut env = Env {
                        blocks: &mut blocks,
                        break_target: None,
                        continue_target: None,
                    };

                    let tail = explicate_tail(bdy, &mut env);
                    env.blocks.insert(sym, tail);

                    fns.insert(sym, FunExplicated {
                        params,
                        blocks,
                        entry,
                    });
                }
                DefAtomized::TypeDef { sym, def } => {
                    defs.insert(sym, def);
                }
            }
        }

        PrgExplicated {
            fns,
            defs,
            entry: self.entry,
        }
    }
}
