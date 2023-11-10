use crate::passes::assign::{Arg, X86Assigned};
use crate::passes::select::{Block, Instr, VarArg, X86Selected};
use crate::utils::gen_sym::UniqueSym;
use crate::*;
use std::collections::HashMap;

impl<'p> X86Selected<'p> {
    #[must_use]
    pub fn assign(self) -> X86Assigned<'p> {
        let program = self.include_liveness();
        let interference = program.compute_interference();
        let (color_map, stack_space) = interference.color();

        let blocks = program
            .blocks
            .into_iter()
            .map(|(lbl, block)| (lbl, assign_block(block.into(), &color_map)))
            .collect();

        X86Assigned {
            blocks,
            entry: program.entry,
            stack_space,
            std: program.std,
        }
    }
}

fn assign_block<'p>(
    block: Block<'p, VarArg>,
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
    instr: Instr<'p, VarArg>,
    color_map: &HashMap<UniqueSym, Arg>,
) -> Instr<'p, Arg> {
    let map = |arg: VarArg| -> Arg {
        match arg {
            VarArg::Imm { val } => Arg::Imm { val },
            VarArg::Reg { reg } => Arg::Reg { reg },
            VarArg::Deref { reg, off } => Arg::Deref { reg, off },
            VarArg::XVar { sym } => color_map[&sym].clone(),
        }
    };

    match instr {
        Instr::Addq { src, dst } => addq!(map(src), map(dst)),
        Instr::Subq { src, dst } => subq!(map(src), map(dst)),
        Instr::Divq { divisor } => divq!(map(divisor)),
        Instr::Mulq { src } => mulq!(map(src)),
        Instr::Negq { dst } => negq!(map(dst)),
        Instr::Movq { src, dst } => movq!(map(src), map(dst)),
        Instr::Pushq { src } => pushq!(map(src)),
        Instr::Popq { dst } => popq!(map(dst)),
        Instr::CallqDirect { lbl, arity } => callq_direct!(lbl, arity),
        Instr::Retq => retq!(),
        Instr::Syscall { arity } => syscall!(arity),
        Instr::Jmp { lbl } => jmp!(lbl),
        Instr::Jcc { lbl, cnd } => jcc!(lbl, cnd),
        Instr::Cmpq { src, dst } => cmpq!(map(src), map(dst)),
        Instr::Andq { src, dst } => andq!(map(src), map(dst)),
        Instr::Orq { src, dst } => orq!(map(src), map(dst)),
        Instr::Xorq { src, dst } => xorq!(map(src), map(dst)),
        Instr::Notq { dst } => notq!(map(dst)),
        Instr::Setcc { cnd } => setcc!(cnd),
        Instr::LoadLbl { sym, dst } => load_lbl!(sym, map(dst)),
        Instr::CallqIndirect { src, arity } => callq_indirect!(map(src), arity),
    }
}
