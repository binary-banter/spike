use crate::language::x86var::{Block, Instr, LArg, LBlock, LX86VarProgram, VarArg, X86VarProgram, ARG_PASSING_REGS, CALLER_SAVED, SYSCALL_REGS, Reg};
use std::collections::{HashMap, HashSet};
use crate::passes::uniquify::UniqueSym;

impl<'p> X86VarProgram<'p> {
    pub fn add_liveness(self) -> LX86VarProgram<'p> {
        let mut before_map = HashMap::new(); //TODO
        LX86VarProgram {
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| (name, block_liveness(block, &before_map)))
                .collect(),
            entry: self.entry,
            std: self.std,
        }
    }
}

fn block_liveness<'p>(block: Block<'p, VarArg<'p>>, before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>) -> LBlock<'p> {
    let mut instrs = Vec::new();
    let mut live = HashSet::new();

    for instr in block.instrs.into_iter().rev() {
        let last_live = live.clone();

        handle_instr(&instr, before_map, |arg, op| {
            match (arg, op) {
                (VarArg::Imm { .. }, _) => {}
                (VarArg::Reg { reg }, ReadWriteOp::Read) => {
                    live.insert(LArg::Reg { reg: *reg });
                },
                (VarArg::Reg { .. } | VarArg::XVar { .. }, ReadWriteOp::ReadWrite) => {},
                (VarArg::Reg { reg }, ReadWriteOp::Write) => {
                    live.remove(&LArg::Reg { reg: *reg });
                },
                (VarArg::XVar { sym }, ReadWriteOp::Read) => {
                    live.insert(LArg::Var { sym: *sym });
                },
                (VarArg::XVar { sym }, ReadWriteOp::Write) => {
                    live.remove(&LArg::Var { sym: *sym });
                },
                (VarArg::Deref { reg, .. }, _) => {
                    live.insert(LArg::Reg { reg: *reg });
                }
            }
        });

        instrs.push((instr, last_live));
    }

    instrs.reverse();
    LBlock { instrs }
}

enum ReadWriteOp {
    Read,
    Write,
    ReadWrite,
}

fn handle_instr<'p>(instr:  &Instr<'p, VarArg<'p>>, before_map: &HashMap<UniqueSym<'p>, HashSet<LArg<'p>>>, mut arg: impl FnMut(&VarArg<'p>, ReadWriteOp)) {
    use ReadWriteOp::Read as R;
    use ReadWriteOp::Write as W;
    use ReadWriteOp::ReadWrite as RW;

    match instr {
        Instr::Addq { src, dst }
        | Instr::Subq { src, dst }
        | Instr::Cmpq { src, dst }
        | Instr::Andq { src, dst }
        | Instr::Orq { src, dst }
        | Instr::Xorq { src, dst } => {
            arg(dst, RW);
            arg(src, R);
        },
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
        Instr::Callq { arity, .. } => {
            for reg in CALLER_SAVED {
                arg(&VarArg::Reg {reg }, W);
            }
            for reg in ARG_PASSING_REGS.into_iter().take(*arity) {
                arg(&VarArg::Reg {reg}, R);
            }
        }
        Instr::Syscall { arity } => {
            for reg in CALLER_SAVED {
                arg(&VarArg::Reg {reg }, W);
            }
            for reg in SYSCALL_REGS.into_iter().take(*arity) {
                arg(&VarArg::Reg {reg}, R);
            }
        }
        Instr::Retq => {
            // Because the return value of our function is in RAX, we need to consider it being read at the end of a block.
            arg(&VarArg::Reg {reg: Reg::RAX}, R);
        },
        Instr::Setcc { .. } => {
            arg(&VarArg::Reg {reg: Reg::RAX}, W);
        },
        Instr::Mulq { src } => {
            arg(&VarArg::Reg {reg: Reg::RDX}, W);
            arg(&VarArg::Reg {reg: Reg::RAX}, RW);
            arg(&src, R);
        }
        Instr::Divq { divisor } => {
            arg(&VarArg::Reg {reg: Reg::RDX}, RW);
            arg(&VarArg::Reg {reg: Reg::RAX}, RW);
            arg(&divisor, R);
        },
        Instr::Jmp { lbl } | Instr::Jcc { lbl, .. } => {
            for larg in &before_map[lbl] {
                arg(&(*larg).into(), R);
            }
        },
    }
}
