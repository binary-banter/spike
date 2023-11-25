use crate::passes::eliminate::eliminate_params::eliminate_params;
use crate::passes::eliminate::eliminate_tail::eliminate_tail;
use crate::passes::eliminate::PrgEliminated;
use crate::passes::explicate::PrgExplicated;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

// (Old variable name, field name) -> New variable name
pub type Ctx<'p> = HashMap<(UniqueSym<'p>, &'p str), UniqueSym<'p>>;

impl<'p> PrgExplicated<'p> {
    pub fn eliminate(self) -> PrgEliminated<'p> {
        let mut ctx = Ctx::new();

        let fn_params = eliminate_params(self.fn_params, &mut ctx, &self.defs);

        PrgEliminated {
            blocks: self
                .blocks
                .into_iter()
                .map(|(sym, tail)| (sym, eliminate_tail(tail, &mut ctx, &self.defs)))
                .collect(),
            fn_params,
            defs: self.defs,
            entry: self.entry,
            std: self.std,
        }
    }
}
