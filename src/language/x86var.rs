use crate::passes::uniquify::UniqueSym;
use crate::{addq, callq, jmp, movq, negq, popq, pushq, retq, subq};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct X86Program<'p> {
    pub blocks: HashMap<&'p str, Block<'p, Arg>>,
}

#[derive(Debug, PartialEq)]
pub struct PX86Program<'p> {
    pub blocks: HashMap<&'p str, Block<'p, Arg>>,
    pub stack_space: usize,
}

#[derive(Debug, PartialEq)]
pub struct AX86Program<'p> {
    pub blocks: HashMap<&'p str, Block<'p, Arg>>,
    pub stack_space: usize,
}

#[derive(Debug, PartialEq)]
pub struct X86VarProgram<'p> {
    pub blocks: HashMap<&'p str, Block<'p, VarArg<'p>>>,
}

pub struct LX86VarProgram<'p> {
    pub blocks: HashMap<&'p str, LBlock<'p>>,
}

#[derive(Debug, PartialEq)]
pub struct Block<'p, A> {
    pub instrs: Vec<Instr<'p, A>>,
}

#[derive(Debug, PartialEq)]
pub struct LBlock<'p> {
    pub instrs: Vec<(Instr<'p, VarArg<'p>>, HashSet<LArg<'p>>)>,
}

#[derive(Debug, PartialEq)]
pub enum Instr<'p, A> {
    Addq { src: A, dst: A },
    Subq { src: A, dst: A },
    Negq { dst: A },
    Movq { src: A, dst: A },
    Pushq { src: A },
    Popq { dst: A },
    Callq { lbl: &'p str, arity: usize },
    Retq,
    Jmp { lbl: &'p str },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum VarArg<'p> {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
    XVar { sym: UniqueSym<'p> },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum Arg {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
}

#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum LArg<'p> {
    Var { sym: UniqueSym<'p> },
    Reg { reg: Reg },
}

pub const CALLER_SAVED: [Reg; 9] = [
    Reg::RAX,
    Reg::RCX,
    Reg::RDX,
    Reg::RSI,
    Reg::RDI,
    Reg::R8,
    Reg::R9,
    Reg::R10,
    Reg::R11,
];
pub const CALLEE_SAVED: [Reg; 7] = [
    Reg::RSP,
    Reg::RBP,
    Reg::RBX,
    Reg::R12,
    Reg::R13,
    Reg::R14,
    Reg::R15,
];
pub const ARG_PASSING_REGS: [Reg; 6] = [Reg::RDI, Reg::RSI, Reg::RDX, Reg::RCX, Reg::R8, Reg::R9];

/// caller-saved:   rax rcx rdx rsi rdi r8 r9 r10 r11
/// callee-saved:   rsp rbp rbx r12 r13 r14 r15
/// arg-passing:    rdi rsi rdx rcx r8 r9
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
pub enum Reg {
    RSP,
    RBP,
    RAX,
    RBX,
    RCX,
    RDX,
    RSI,
    RDI,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl<'p> From<X86Program<'p>> for X86VarProgram<'p> {
    fn from(value: X86Program<'p>) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl<'p> From<PX86Program<'p>> for X86VarProgram<'p> {
    fn from(value: PX86Program<'p>) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl<'p> From<AX86Program<'p>> for X86VarProgram<'p> {
    fn from(value: AX86Program<'p>) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl<'p> From<Block<'p, Arg>> for Block<'p, VarArg<'p>> {
    fn from(value: Block<'p, Arg>) -> Self {
        Block {
            instrs: value.instrs.into_iter().map(From::from).collect(),
        }
    }
}

impl<'p> From<Instr<'p, Arg>> for Instr<'p, VarArg<'p>> {
    fn from(value: Instr<'p, Arg>) -> Self {
        match value {
            Instr::Addq { src, dst } => addq!(src.into(), dst.into()),
            Instr::Subq { src, dst } => subq!(src.into(), dst.into()),
            Instr::Negq { dst } => negq!(dst.into()),
            Instr::Movq { src, dst } => movq!(src.into(), dst.into()),
            Instr::Pushq { src } => pushq!(src.into()),
            Instr::Popq { dst } => popq!(dst.into()),
            Instr::Callq { lbl, arity } => callq!(lbl, arity),
            Instr::Retq => retq!(),
            Instr::Jmp { lbl } => jmp!(lbl),
        }
    }
}

impl<'p> From<Arg> for VarArg<'p> {
    fn from(value: Arg) -> Self {
        match value {
            Arg::Imm { val } => VarArg::Imm { val },
            Arg::Reg { reg } => VarArg::Reg { reg },
            Arg::Deref { reg, off } => VarArg::Deref { reg, off },
        }
    }
}

mod macros {
    #[macro_export]
    macro_rules! block {
        ($($instr:expr),*) => {
            Block { instrs: vec![$($instr),*] }
        };
    }

    #[macro_export]
    macro_rules! addq {
        ($src:expr, $dst:expr) => {
            Instr::Addq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! subq {
        ($src:expr, $dst:expr) => {
            Instr::Subq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! negq {
        ($dst:expr) => {
            Instr::Negq { dst: $dst }
        };
    }

    #[macro_export]
    macro_rules! movq {
        ($src:expr, $dst:expr) => {
            Instr::Movq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! pushq {
        ($src:expr) => {
            Instr::Pushq { src: $src }
        };
    }

    #[macro_export]
    macro_rules! popq {
        ($dst:expr) => {
            Instr::Popq { dst: $dst }
        };
    }

    #[macro_export]
    macro_rules! callq {
        ($lbl:expr, $arity:expr) => {
            Instr::Callq {
                lbl: $lbl,
                arity: $arity,
            }
        };
    }

    #[macro_export]
    macro_rules! jmp {
        ($lbl:expr) => {
            Instr::Jmp { lbl: $lbl }
        };
    }

    #[macro_export]
    macro_rules! retq {
        () => {
            Instr::Retq
        };
    }

    #[macro_export]
    macro_rules! imm {
        ($val:expr) => {
            Arg::Imm { val: $val }.into()
        };
    }

    #[macro_export]
    macro_rules! reg {
        ($reg:ident) => {
            Arg::Reg { reg: Reg::$reg }.into()
        };
    }

    #[macro_export]
    macro_rules! var {
        ($sym:expr) => {
            VarArg::XVar { sym: $sym }
        };
    }
}
