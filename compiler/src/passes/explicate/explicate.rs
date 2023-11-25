use crate::passes::atomize::PrgAtomized;
use crate::passes::explicate::{CTail, explicate_def, PrgExplicated};
use crate::passes::parse::Def;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub struct Env<'a, 'p> {
    pub blocks: &'a mut HashMap<UniqueSym<'p>, CTail<'p>>,
    /// (block to jump to, variable to write to)
    pub break_target: Option<(UniqueSym<'p>, UniqueSym<'p>)>,
    /// block to jump to
    pub continue_target: Option<UniqueSym<'p>>,
}

impl<'p> PrgAtomized<'p> {
    #[must_use]
    pub fn explicate(self) -> PrgExplicated<'p> {
        let mut blocks = HashMap::new();
        let mut env = Env {
            blocks: &mut blocks,
            break_target: None,
            continue_target: None,
        };

        let mut fn_params = HashMap::new();

        for (sym, def) in &self.defs {
            match def {
                Def::Fn { params, .. } => {
                    fn_params.insert(*sym, params.clone());
                }
                Def::TypeDef { .. } => {
                    // todo?
                }
            }
        }

        let mut defs = HashMap::new();

        for (_, def) in self.defs {
            explicate_def::explicate_def(def, &mut env, &mut defs);
        }

        PrgExplicated {
            blocks,
            fn_params,
            defs,
            entry: self.entry,
            std: self.std
        }
    }
}
