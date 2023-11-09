use crate::passes::interference::{Arg, X86Assigned, X86Colored};
use crate::passes::select::{Block, Instr, VarArg};
use crate::utils::gen_sym::UniqueSym;
use crate::{
    addq, andq, callq_direct, callq_indirect, cmpq, divq, jcc, jmp, load_lbl, movq, mulq, negq,
    notq, orq, popq, pushq, retq, setcc, subq, syscall, xorq,
};
use std::collections::HashMap;

impl<'p> X86Colored<'p> {
    #[must_use]
    pub fn assign_homes(self) -> X86Assigned<'p> {
        X86Assigned {
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| {
                    (
                        name,
                        Block {
                            instrs: block
                                .instrs
                                .into_iter()
                                .map(|instr| assign_instr(instr, &self.color_map))
                                .collect(),
                        },
                    )
                })
                .collect(),
            entry: self.entry,
            stack_space: self.stack_space,
            std: self.std,
        }
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
            VarArg::XVar { sym } => color_map[&sym],
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
