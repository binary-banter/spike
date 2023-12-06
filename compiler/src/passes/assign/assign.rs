use crate::passes::assign::{Arg, FunAssigned, X86Assigned};
use crate::passes::select::{Block, Instr, InstrSelected, VarArg, X86Selected};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::time::time;
use functor_derive::Functor;
use std::collections::HashMap;

impl<'p> X86Selected<'p> {
    #[must_use]
    pub fn assign(self) -> X86Assigned<'p> {
        time("select");

        let program = self.include_liveness();
        let fns = program.fns.fmap(|fun| {
            let interference = fun.compute_interference();
            let (color_map, stack_space) = interference.color();

            let blocks = fun
                .blocks
                .into_iter()
                .map(|(lbl, block)| (lbl, assign_block(block.into(), &color_map)))
                .collect();

            FunAssigned {
                blocks,
                entry: fun.entry,
                exit: fun.exit,
                stack_space,
            }
        });

        X86Assigned {
            fns,
            entry: program.entry,
        }
    }
}

fn assign_block<'p>(
    block: Block<'p, VarArg<UniqueSym<'p>>>,
    color_map: &HashMap<UniqueSym, Arg>,
) -> Block<'p, Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .map(|instr| assign_instr(instr, color_map))
            .collect(),
    }
}

fn assign_instr<'p>(
    instr: InstrSelected<'p>,
    color_map: &HashMap<UniqueSym, Arg>,
) -> Instr<Arg, UniqueSym<'p>> {
    let map = |arg: VarArg<UniqueSym<'p>>| -> Arg {
        match arg {
            VarArg::Imm(imm) => Arg::Imm(imm),
            VarArg::Reg(reg) => Arg::Reg(reg),
            VarArg::Deref { reg, off } => Arg::Deref { reg, off },
            VarArg::XVar(sym) => color_map[&sym].clone(),
        }
    };
    instr.fmap(map)
}
