use crate::language::x86var::{
    Block, Instr, LArg, LBlock, LX86VarProgram, Reg, VarArg, X86VarProgram, ARG_PASSING_REGS,
    CALLER_SAVED,
};
use crate::reg;
use std::collections::HashSet;

impl<'p> X86VarProgram<'p> {
    pub fn add_liveness(self) -> LX86VarProgram<'p> {
        LX86VarProgram {
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| (name, block_liveness(block)))
                .collect(),
        }
    }
}

fn block_liveness<'p>(block: Block<'p, VarArg<'p>>) -> LBlock<'p> {
    let mut instrs = Vec::new();
    let mut live = HashSet::new();

    for instr in block.instrs.into_iter().rev() {
        let last_live = live.clone();

        for arg in instr_writes(&instr) {
            live.remove(&arg);
        }

        for arg in instr_reads(&instr) {
            live.insert(arg);
        }

        instrs.push((instr, last_live));
    }

    instrs.reverse();
    LBlock { instrs }
}

fn instr_reads<'p>(instr: &Instr<'p, VarArg<'p>>) -> impl Iterator<Item = LArg<'p>> {
    let reads = instr_reads_vararg(instr).filter_map(|arg| match arg {
        VarArg::Imm { .. } => None,
        VarArg::Reg { reg } => Some(LArg::Reg { reg }),
        VarArg::Deref { reg, .. } => Some(LArg::Reg { reg }),
        VarArg::XVar { sym } => Some(LArg::Var { sym }),
    });
    let writes = instr_writes_vararg(instr).filter_map(|arg| match arg {
        VarArg::Imm { .. } => None,
        VarArg::Reg { .. } => None,
        VarArg::Deref { reg, .. } => Some(LArg::Reg { reg }),
        VarArg::XVar { .. } => None,
    });

    reads.chain(writes)
}

pub fn instr_writes<'p>(instr: &Instr<'p, VarArg<'p>>) -> impl Iterator<Item = LArg<'p>> {
    instr_writes_vararg(instr).filter_map(|arg| match arg {
        VarArg::Imm { .. } => None,
        VarArg::Reg { reg } => Some(LArg::Reg { reg }),
        VarArg::Deref { .. } => None,
        VarArg::XVar { sym } => Some(LArg::Var { sym }),
    })
}

fn instr_reads_vararg<'p>(instr: &Instr<'p, VarArg<'p>>) -> impl Iterator<Item = VarArg<'p>> {
    match instr {
        Instr::Addq { src, dst } | Instr::Subq { src, dst } => {
            heapless::Vec::from_iter([*src, *dst])
        }
        Instr::Movq { src, .. } | Instr::Pushq { src } | Instr::Negq { dst: src } => {
            heapless::Vec::from_iter([*src])
        }
        Instr::Callq { arity, .. } => heapless::Vec::from_iter(
            ARG_PASSING_REGS
                .iter()
                .take(*arity)
                .map(|&reg| VarArg::Reg { reg }),
        ),
        Instr::Popq { .. } | Instr::Jmp { .. } | Instr::Retq => heapless::Vec::<VarArg, 6>::new(),
        Instr::Divq { divisor } => heapless::Vec::from_iter([*divisor, reg!(RAX), reg!(RDX)]),
        Instr::Mulq { src } => heapless::Vec::from_iter([*src, reg!(RAX)]),
        Instr::Jcc { .. } => todo!(),
        Instr::Syscall => unreachable!("There should be no syscalls in this pass."),
    }
    .into_iter()
}

fn instr_writes_vararg<'p>(instr: &Instr<'p, VarArg<'p>>) -> impl Iterator<Item = VarArg<'p>> {
    match instr {
        Instr::Addq { dst, .. } | Instr::Subq { dst, .. } => heapless::Vec::from_iter([*dst]),
        Instr::Movq { dst, .. } | Instr::Popq { dst } | Instr::Negq { dst } => {
            heapless::Vec::from_iter([*dst])
        }
        Instr::Callq { .. } => {
            heapless::Vec::from_iter(CALLER_SAVED.iter().map(|&reg| VarArg::Reg { reg }))
        }
        Instr::Pushq { .. } | Instr::Jmp { .. } | Instr::Retq => heapless::Vec::<VarArg, 9>::new(),
        Instr::Mulq { .. } | Instr::Divq { .. } => heapless::Vec::from_iter([reg!(RAX), reg!(RDX)]),
        Instr::Jcc { .. } => todo!(),
        Instr::Syscall => unreachable!("There should be no syscalls in this pass."),
    }
    .into_iter()
}
