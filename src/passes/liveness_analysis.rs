use crate::language::x86var::{
    Block, Instr, LArg, LBlock, LX86VarProgram, Reg, VarArg, X86VarProgram, ARG_PASSING_REGS,
};
use std::collections::{HashSet, VecDeque};
use std::iter;

impl X86VarProgram {
    pub fn add_liveness(self) -> LX86VarProgram {
        LX86VarProgram {
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| (name, block_liveness(block)))
                .collect(),
        }
    }
}

fn block_liveness(block: Block<VarArg>) -> LBlock {
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

fn instr_reads(instr: &Instr<VarArg>) -> HashSet<LArg> {
    match instr {
        Instr::Addq { src, dst } | Instr::Subq { src, dst } => [src, dst]
            .into_iter()
            .cloned()
            .map(TryFrom::try_from)
            .flatten()
            .collect(),
        Instr::Movq { src, .. } | Instr::Pushq { src } | Instr::Negq { dst: src } => [src]
            .into_iter()
            .cloned()
            .map(TryFrom::try_from)
            .flatten()
            .collect(),
        Instr::Callq { arity, .. } => ARG_PASSING_REGS
            .iter()
            .take(*arity)
            .cloned()
            .map(|reg| LArg::Reg { reg })
            .collect(),
        Instr::Popq { .. } | Instr::Jmp { .. } | Instr::Retq => HashSet::new(),
    }
}

fn instr_writes(instr: &Instr<VarArg>) -> HashSet<LArg> {
    match instr {
        Instr::Addq { dst, .. } | Instr::Subq { dst, .. } => [dst]
            .into_iter()
            .cloned()
            .map(TryFrom::try_from)
            .flatten()
            .collect(),
        Instr::Movq { dst, .. } | Instr::Popq { dst } | Instr::Negq { dst } => [dst]
            .into_iter()
            .cloned()
            .map(TryFrom::try_from)
            .flatten()
            .collect(),
        Instr::Callq { .. } => HashSet::from([LArg::Reg { reg: Reg::RAX }]),
        Instr::Pushq { .. } | Instr::Jmp { .. } | Instr::Retq => HashSet::new(),
    }
}

impl TryFrom<VarArg> for LArg {
    type Error = ();

    fn try_from(value: VarArg) -> Result<Self, Self::Error> {
        match value {
            VarArg::Imm { .. } => Err(()),
            VarArg::Reg { reg } => Ok(LArg::Reg { reg }),
            VarArg::Deref { reg, .. } => Ok(LArg::Reg { reg }),
            VarArg::XVar { sym } => Ok(LArg::Var { sym: sym.clone() }),
        }
    }
}

//           case Instr(Addq() | Subq(), s1 :: s2d1 :: Nil) => after ++ argToW(s2d1) ++ argToW(s1)
//           case Instr(Movq(), s1 :: d1 :: Nil) => after -- argToW(d1) ++ argToW(s1)
//           case Instr(Negq(), s1d1 :: Nil) => after ++ argToW(s1d1)
//           case Instr(Popq(), d1 :: Nil) => after -- argToW(d1)
//           case Instr(Pushq(), s1 :: Nil) => after ++ argToW(s1)
//           case Callq("_read_int", 0) => after -- Set(Rg(RAX()))
//           case Callq("_print_int", 1) => after ++ Set(Rg(RDI()))
