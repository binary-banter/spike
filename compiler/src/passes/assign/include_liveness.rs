use crate::utils::gen_sym::UniqueSym;

use crate::passes::assign::{LArg, LBlock, LX86VarProgram};
use crate::passes::select::{
    Block, Instr, InstrSelected, Reg, VarArg, X86Selected, CALLER_SAVED, SYSCALL_REGS,
};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

impl<'p> X86Selected<'p> {
    #[must_use]
    pub(super) fn include_liveness(self) -> LX86VarProgram<'p> {
        // maps block names to what is live before the block
        let mut before_map = HashMap::<UniqueSym, HashSet<LArg>>::new();
        // maps block names to blocks with liveness info added
        let mut liveness = HashMap::<UniqueSym, LBlock>::new();

        let mut changed = true;

        while changed {
            changed = false;

            for (sym, block) in &self.blocks {
                let (new_liveness, before) = block_liveness(block, &before_map);

                match before_map.entry(*sym) {
                    Entry::Occupied(mut e) => {
                        if e.get() != &before {
                            changed = true;
                            e.insert(before);
                        }
                    }
                    Entry::Vacant(e) => {
                        changed = true;
                        e.insert(before);
                    }
                }

                match liveness.get(sym) {
                    None => {
                        liveness.insert(*sym, new_liveness);
                        changed = true;
                    }
                    Some(old_liveness) => {
                        if *old_liveness != new_liveness {
                            liveness.insert(*sym, new_liveness);
                            changed = true;
                        }
                    }
                }
            }
        }

        LX86VarProgram {
            blocks: liveness,
            entry: self.entry,
            std: self.std,
        }
    }
}

fn block_liveness<'p>(
    block: &Block<'p, VarArg<UniqueSym<'p>>>,
    before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>,
) -> (LBlock<'p>, HashSet<LArg<'p>>) {
    let mut instrs = Vec::new();
    let mut live = HashSet::new();

    for instr in block.instrs.iter().rev() {
        let last_live = live.clone();

        handle_instr(instr, before_map, |arg, op| match (arg, op) {
            (VarArg::Imm { .. }, _) => {}
            (VarArg::Reg { reg }, ReadWriteOp::Read | ReadWriteOp::ReadWrite) => {
                live.insert(LArg::Reg { reg: *reg });
            }
            (VarArg::Reg { reg }, ReadWriteOp::Write) => {
                live.remove(&LArg::Reg { reg: *reg });
            }
            (VarArg::XVar { sym }, ReadWriteOp::Read | ReadWriteOp::ReadWrite) => {
                live.insert(LArg::Var { sym: *sym });
            }
            (VarArg::XVar { sym }, ReadWriteOp::Write) => {
                live.remove(&LArg::Var { sym: *sym });
            }
            (VarArg::Deref { reg, .. }, _) => {
                live.insert(LArg::Reg { reg: *reg });
            }
        });

        instrs.push((instr.clone(), last_live));
    }

    instrs.reverse();
    (LBlock { instrs }, live)
}

pub enum ReadWriteOp {
    Read,
    Write,
    ReadWrite,
}

pub fn handle_instr<'p>(
    instr: &InstrSelected<'p>,
    before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>,
    mut arg: impl FnMut(&VarArg<UniqueSym<'p>>, ReadWriteOp),
) {
    use ReadWriteOp::Read as R;
    use ReadWriteOp::ReadWrite as RW;
    use ReadWriteOp::Write as W;

    match instr {
        Instr::Addq { src, dst }
        | Instr::Subq { src, dst }
        | Instr::Andq { src, dst }
        | Instr::Orq { src, dst }
        | Instr::Xorq { src, dst } => {
            arg(dst, RW);
            arg(src, R);
        }
        Instr::Cmpq { src, dst } => {
            arg(dst, R);
            arg(src, R);
        }
        Instr::Movq { src, dst } => {
            arg(dst, W);
            arg(src, R);
        }
        Instr::Pushq { src } => {
            arg(src, R);
        }
        Instr::Popq { dst } => {
            arg(dst, W);
        }
        Instr::Negq { dst } | Instr::Notq { dst } => {
            arg(dst, RW);
        }
        Instr::CallqDirect { arity, .. } => {
            for reg in CALLER_SAVED.into_iter().skip(*arity) {
                arg(&VarArg::Reg { reg }, W);
            }
            for reg in CALLER_SAVED.into_iter().take(*arity) {
                arg(&VarArg::Reg { reg }, RW);
            }
        }
        Instr::Syscall { arity } => {
            for reg in CALLER_SAVED {
                arg(&VarArg::Reg { reg }, W);
            }
            for reg in SYSCALL_REGS.into_iter().take(*arity) {
                arg(&VarArg::Reg { reg }, R);
            }
        }
        Instr::Retq => {
            // Because the return value of our function is in RAX, we need to consider it being read at the end of a block.
            arg(&VarArg::Reg { reg: Reg::RAX }, R);
        }
        Instr::Setcc { .. } => {
            arg(&VarArg::Reg { reg: Reg::RAX }, W);
        }
        Instr::Mulq { src } => {
            arg(&VarArg::Reg { reg: Reg::RDX }, W);
            arg(&VarArg::Reg { reg: Reg::RAX }, RW);
            arg(src, R);
        }
        Instr::Divq { divisor } => {
            arg(&VarArg::Reg { reg: Reg::RDX }, RW);
            arg(&VarArg::Reg { reg: Reg::RAX }, RW);
            arg(divisor, R);
        }
        Instr::Jmp { lbl } | Instr::Jcc { lbl, .. } => {
            for larg in before_map.get(lbl).unwrap_or(&HashSet::new()) {
                arg(&(*larg).into(), R);
            }
        }
        Instr::LoadLbl { dst, .. } => {
            arg(dst, W);
        }
        Instr::CallqIndirect { src, arity } => {
            for reg in CALLER_SAVED.into_iter().skip(*arity) {
                arg(&VarArg::Reg { reg }, W);
            }
            for reg in CALLER_SAVED.into_iter().take(*arity) {
                arg(&VarArg::Reg { reg }, RW);
            }
            arg(src, R);
        }
    }
}
