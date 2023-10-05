use crate::{addq, callq, jmp, movq, negq, popq, pushq, retq, subq};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct X86Program {
    pub blocks: HashMap<String, Block<Arg>>,
}

#[derive(Debug, PartialEq)]
pub struct PX86Program {
    pub blocks: HashMap<String, Block<Arg>>,
    pub stack_space: usize,
}

#[derive(Debug, PartialEq)]
pub struct AX86Program {
    pub blocks: HashMap<String, Block<Arg>>,
    pub stack_space: usize,
}

#[derive(Debug, PartialEq)]
pub struct X86VarProgram {
    pub blocks: HashMap<String, Block<VarArg>>,
}

#[derive(Debug, PartialEq)]
pub struct Block<A> {
    pub instrs: Vec<Instr<A>>,
}

#[derive(Debug, PartialEq)]
pub enum Instr<A> {
    Addq { src: A, dst: A },
    Subq { src: A, dst: A },
    Negq { dst: A },
    Movq { src: A, dst: A },
    Pushq { src: A },
    Popq { dst: A },
    Callq { lbl: String, arity: usize },
    Retq,
    Jmp { lbl: String },
}

#[derive(Debug, PartialEq)]
pub enum SysOp {
    Exit,
}

#[derive(Debug, PartialEq, Clone)]
pub enum VarArg {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
    XVar { sym: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Arg {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
}

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

impl From<X86Program> for X86VarProgram {
    fn from(value: X86Program) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl From<PX86Program> for X86VarProgram {
    fn from(value: PX86Program) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl From<AX86Program> for X86VarProgram {
    fn from(value: AX86Program) -> Self {
        X86VarProgram {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
        }
    }
}

impl From<Block<Arg>> for Block<VarArg> {
    fn from(value: Block<Arg>) -> Self {
        Block {
            instrs: value.instrs.into_iter().map(From::from).collect(),
        }
    }
}

impl From<Instr<Arg>> for Instr<VarArg> {
    fn from(value: Instr<Arg>) -> Self {
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

impl From<Arg> for VarArg {
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
                lbl: $lbl.to_string(),
                arity: $arity,
            }
        };
    }

    #[macro_export]
    macro_rules! jmp {
        ($lbl:expr) => {
            Instr::Jmp {
                lbl: $lbl.to_string(),
            }
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
