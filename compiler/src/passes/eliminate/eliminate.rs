use crate::passes::eliminate::eliminate_params::eliminate_params;
use crate::passes::eliminate::eliminate_tail::eliminate_tail;
use crate::passes::eliminate::{FunEliminated, PrgEliminated};
use crate::passes::explicate::PrgExplicated;
use crate::time;
use crate::utils::unique_sym::UniqueSym;
use std::collections::HashMap;

// (Old variable name, field name) -> New variable name
pub type Ctx<'p> = HashMap<(UniqueSym<'p>, &'p str), UniqueSym<'p>>;

impl<'p> PrgExplicated<'p> {
    pub fn eliminate(self) -> PrgEliminated<'p> {
        let fns = self
            .fns
            .into_iter()
            .map(|(sym, fun)| {
                let mut ctx = Ctx::new();

                let fun = FunEliminated {
                    params: eliminate_params(fun.params, &mut ctx, &self.defs),
                    blocks: fun
                        .blocks
                        .into_iter()
                        .map(|(sym, tail)| (sym, eliminate_tail(tail, &mut ctx, &self.defs)))
                        .collect(),
                    entry: fun.entry,
                };

                (sym, fun)
            })
            .collect();

        let program = PrgEliminated {
            fns,
            defs: self.defs,
            entry: self.entry,
        };

        // display!(&program, Eliminate); // todo
        time!("eliminate");

        program
    }
}
