use crate::passes::atomize::{DefAtomized, PrgAtomized};
use crate::passes::explicate::explicate_tail::explicate_tail;
use crate::passes::explicate::{FunExplicated, PrgExplicated, TailExplicated};
use crate::utils::unique_sym::UniqueSym;
use crate::{display, time};
use std::collections::HashMap;

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

        for (def_sym, def) in self.defs {
            match def {
                DefAtomized::Fn {
                    sym, params, bdy, ..
                } => {
                    let mut blocks = HashMap::new();
                    let mut env = Env {
                        blocks: &mut blocks,
                        break_target: None,
                        continue_target: None,
                    };

                    let tail = explicate_tail(bdy, &mut env);
                    let entry = sym.fresh();
                    env.blocks.insert(entry, tail);

                    fns.insert(
                        def_sym,
                        FunExplicated {
                            params,
                            blocks,
                            entry,
                        },
                    );
                }
                DefAtomized::TypeDef { sym, def } => {
                    defs.insert(sym, def);
                }
            }
        }

        let program = PrgExplicated {
            fns,
            defs,
            entry: self.entry,
        };

        display!(&program, Reveal);
        time!("reveal");

        program
    }
}
