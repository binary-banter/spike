use crate::passes::select::io::Std;
use crate::passes::uniquify::UniqueSym;
use crate::{
    addq, andq, callq_direct, callq_indirect, cmpq, divq, jcc, jmp, load_lbl, movq, mulq, negq,
    notq, orq, popq, pushq, retq, setcc, subq, syscall, xorq,
};
use petgraph::prelude::GraphMap;
use petgraph::Undirected;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Debug, PartialEq)]
pub struct X86Concluded<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq)]
pub struct X86Patched<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub stack_space: usize,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq)]
pub struct X86Assigned<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub stack_space: usize,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq)]
pub struct X86Selected<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq)]
pub struct LX86VarProgram<'p> {
    pub blocks: HashMap<UniqueSym<'p>, LBlock<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug)]
pub struct X86WithInterference<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub interference: InterferenceGraph<'p>,
    pub std: Std<'p>,
}

#[derive(Debug)]
pub struct X86Colored<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub color_map: HashMap<UniqueSym<'p>, Arg>,
    pub stack_space: usize,
    pub std: Std<'p>,
}

pub type InterferenceGraph<'p> = GraphMap<LArg<'p>, (), Undirected>;

#[derive(Debug, PartialEq, Clone)]
pub struct Block<'p, A> {
    pub instrs: Vec<Instr<'p, A>>,
}

#[derive(Debug, PartialEq)]
pub struct LBlock<'p> {
    pub instrs: Vec<(Instr<'p, VarArg<'p>>, HashSet<LArg<'p>>)>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cnd {
    Above,
    AboveOrEqual,
    Below,
    BelowOrEqual,
    Carry,
    EQ,
    GT,
    GE,
    LT,
    LE,
    NotCarry,
    NE,
    NotOverflow,
    NotSign,
    Overflow,
    ParityEven,
    ParityOdd,
    Sign,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instr<'p, A> {
    Addq { src: A, dst: A },
    Subq { src: A, dst: A },
    Divq { divisor: A },
    Mulq { src: A },
    Negq { dst: A },
    Movq { src: A, dst: A },
    Pushq { src: A },
    Popq { dst: A },
    Retq,
    Syscall { arity: usize },
    Cmpq { src: A, dst: A },
    Jmp { lbl: UniqueSym<'p> },
    Jcc { lbl: UniqueSym<'p>, cnd: Cnd },
    Andq { src: A, dst: A },
    Orq { src: A, dst: A },
    Xorq { src: A, dst: A },
    Notq { dst: A },
    Setcc { cnd: Cnd }, //TODO allow setting other byteregs
    LoadLbl { sym: UniqueSym<'p>, dst: A },
    CallqDirect { lbl: UniqueSym<'p>, arity: usize },
    CallqIndirect { src: A, arity: usize },
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum VarArg<'p> {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
    XVar { sym: UniqueSym<'p> },
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum Arg {
    Imm { val: i64 },
    Reg { reg: Reg },
    Deref { reg: Reg, off: i64 },
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Ord, PartialOrd)]
pub enum LArg<'p> {
    Var { sym: UniqueSym<'p> },
    Reg { reg: Reg },
}

impl<'p> From<LArg<'p>> for VarArg<'p> {
    fn from(val: LArg<'p>) -> Self {
        match val {
            LArg::Var { sym } => VarArg::XVar { sym },
            LArg::Reg { reg } => VarArg::Reg { reg },
        }
    }
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
pub const CALLEE_SAVED_NO_STACK: [Reg; 5] = [Reg::RBX, Reg::R12, Reg::R13, Reg::R14, Reg::R15];

pub const ARG_PASSING_REGS: [Reg; 6] = [Reg::RDI, Reg::RSI, Reg::RDX, Reg::RCX, Reg::R8, Reg::R9];
pub const SYSCALL_REGS: [Reg; 7] = [
    Reg::RAX,
    Reg::RDI,
    Reg::RSI,
    Reg::RDX,
    Reg::RCX,
    Reg::R8,
    Reg::R9,
];

/// caller-saved:   rax rcx rdx rsi rdi r8 r9 r10 r11
/// callee-saved:   rsp rbp rbx r12 r13 r14 r15
/// arg-passing:    rdi rsi rdx rcx r8 r9
#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Ord, PartialOrd)]
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

impl<'p> From<X86Concluded<'p>> for X86Selected<'p> {
    fn from(value: X86Concluded<'p>) -> Self {
        X86Selected {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
            entry: value.entry,
            std: value.std,
        }
    }
}

impl<'p> From<X86Patched<'p>> for X86Selected<'p> {
    fn from(value: X86Patched<'p>) -> Self {
        X86Selected {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
            entry: value.entry,
            std: value.std,
        }
    }
}

impl<'p> From<X86Assigned<'p>> for X86Selected<'p> {
    fn from(value: X86Assigned<'p>) -> Self {
        X86Selected {
            blocks: value
                .blocks
                .into_iter()
                .map(|(n, b)| (n, b.into()))
                .collect(),
            entry: value.entry,
            std: value.std,
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

impl<'p> From<LBlock<'p>> for Block<'p, VarArg<'p>> {
    fn from(value: LBlock<'p>) -> Self {
        Block {
            instrs: value.instrs.into_iter().map(|(instr, _)| instr).collect(),
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
            Instr::CallqDirect { lbl, arity } => callq_direct!(lbl, arity),
            Instr::Retq => retq!(),
            Instr::Jmp { lbl } => jmp!(lbl),
            Instr::Syscall { arity } => syscall!(arity),
            Instr::Divq { divisor } => divq!(divisor.into()),
            Instr::Jcc { lbl, cnd } => jcc!(lbl, cnd),
            Instr::Mulq { src } => mulq!(src.into()),
            Instr::Cmpq { src, dst } => cmpq!(src.into(), dst.into()),
            Instr::Andq { src, dst } => andq!(src.into(), dst.into()),
            Instr::Orq { src, dst } => orq!(src.into(), dst.into()),
            Instr::Xorq { src, dst } => xorq!(src.into(), dst.into()),
            Instr::Notq { dst } => notq!(dst.into()),
            Instr::Setcc { cnd } => setcc!(cnd),
            Instr::LoadLbl { sym, dst } => load_lbl!(sym, dst.into()),
            Instr::CallqIndirect { src, arity } => callq_indirect!(src.into(), arity),
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
            $crate::language::x86var::Block { instrs: vec![$($instr),*] }
        };
    }

    #[macro_export]
    macro_rules! addq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Addq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! divq {
        ($divisor:expr) => {
            $crate::language::x86var::Instr::Divq { divisor: $divisor }
        };
    }

    #[macro_export]
    macro_rules! mulq {
        ($src:expr) => {
            $crate::language::x86var::Instr::Mulq { src: $src }
        };
    }

    #[macro_export]
    macro_rules! subq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Subq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! cmpq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Cmpq {
                src: $src,
                dst: $dst,
            }
        };
    }
    #[macro_export]
    macro_rules! andq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Andq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! orq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Orq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! xorq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Xorq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! notq {
        ($dst:expr) => {
            $crate::language::x86var::Instr::Notq { dst: $dst }
        };
    }

    #[macro_export]
    macro_rules! negq {
        ($dst:expr) => {
            $crate::language::x86var::Instr::Negq { dst: $dst }
        };
    }

    #[macro_export]
    macro_rules! movq {
        ($src:expr, $dst:expr) => {
            $crate::language::x86var::Instr::Movq {
                src: $src,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! load_lbl {
        ($lbl:expr, $dst: expr) => {
            $crate::language::x86var::Instr::LoadLbl {
                sym: $lbl,
                dst: $dst,
            }
        };
    }

    #[macro_export]
    macro_rules! pushq {
        ($src:expr) => {
            $crate::language::x86var::Instr::Pushq { src: $src }
        };
    }

    #[macro_export]
    macro_rules! popq {
        ($dst:expr) => {
            $crate::language::x86var::Instr::Popq { dst: $dst }
        };
    }

    #[macro_export]
    macro_rules! callq_direct {
        ($lbl:expr, $arity:expr) => {
            $crate::language::x86var::Instr::CallqDirect {
                lbl: $lbl,
                arity: $arity,
            }
        };
    }

    #[macro_export]
    macro_rules! callq_indirect {
        ($src:expr, $arity:expr) => {
            $crate::language::x86var::Instr::CallqIndirect {
                src: $src,
                arity: $arity,
            }
        };
    }

    #[macro_export]
    macro_rules! jmp {
        ($lbl:expr) => {
            $crate::language::x86var::Instr::Jmp { lbl: $lbl }
        };
    }

    #[macro_export]
    macro_rules! setcc {
        ($cnd:expr) => {
            $crate::language::x86var::Instr::Setcc { cnd: $cnd }
        };
    }

    #[macro_export]
    macro_rules! jcc {
        ($lbl:expr, $cnd:expr) => {
            $crate::language::x86var::Instr::Jcc {
                lbl: $lbl,
                cnd: $cnd,
            }
        };
    }

    #[macro_export]
    macro_rules! retq {
        () => {
            $crate::language::x86var::Instr::Retq
        };
    }

    #[macro_export]
    macro_rules! syscall {
        ($arity:expr) => {
            $crate::language::x86var::Instr::Syscall { arity: $arity }
        };
    }

    #[macro_export]
    macro_rules! imm {
        ($val:expr) => {
            $crate::language::x86var::Arg::Imm { val: $val.into() }.into()
        };
    }

    #[macro_export]
    macro_rules! reg {
        ($reg:ident) => {
            $crate::language::x86var::Arg::Reg {
                reg: $crate::language::x86var::Reg::$reg,
            }
            .into()
        };
    }

    #[macro_export]
    macro_rules! var {
        ($sym:expr) => {
            $crate::language::x86var::VarArg::XVar { sym: $sym }
        };
    }

    #[macro_export]
    macro_rules! deref {
        ($reg:ident, $off:expr) => {
            $crate::language::x86var::Arg::Deref {
                reg: Reg::$reg,
                off: $off,
            }
            .into()
        };
    }
}
