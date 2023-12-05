mod display;
pub mod macros;
pub mod select;

use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

pub struct X86Selected<'p> {
    pub fns: HashMap<UniqueSym<'p>, FunSelected<'p>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug)]
pub struct FunSelected<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
    pub entry: UniqueSym<'p>,
    pub exit: UniqueSym<'p>,
}

#[derive(Debug, Clone, Display, Functor)]
#[display(fmt = "\t{}", r#"instrs.iter().format("\n\t")"#)]
pub struct Block<'p, A: Display> {
    pub instrs: Vec<Instr<A, UniqueSym<'p>>>,
}

pub type InstrSelected<'p> = Instr<VarArg<UniqueSym<'p>>, UniqueSym<'p>>;

#[derive(Debug, Copy, Clone, PartialEq, Display)]
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

#[derive(Debug, Clone, PartialEq, Display, Functor)]
pub enum Instr<Arg: Display, IdentVars: Display> {
    #[display(fmt = "addq\t{src} {dst}")]
    Addq { src: Arg, dst: Arg },
    #[display(fmt = "subq\t{src} {dst}")]
    Subq { src: Arg, dst: Arg },
    #[display(fmt = "divq\t{divisor}")]
    Divq { divisor: Arg },
    #[display(fmt = "mulq\t{src}")]
    Mulq { src: Arg },
    #[display(fmt = "negq\t{dst}")]
    Negq { dst: Arg },
    #[display(fmt = "movq\t{src} {dst}")]
    Movq { src: Arg, dst: Arg },
    #[display(fmt = "pushq\t{src}")]
    Pushq { src: Arg },
    #[display(fmt = "popq\t{dst}")]
    Popq { dst: Arg },
    #[display(fmt = "retq")]
    Retq,
    #[display(fmt = "syscall\t{arity}")]
    Syscall { arity: usize },
    #[display(fmt = "cmpq\t{src} {dst}")]
    Cmpq { src: Arg, dst: Arg },
    #[display(fmt = "jmp\t{lbl}")]
    Jmp { lbl: IdentVars },
    #[display(fmt = "jcc\t{cnd} {lbl}")]
    Jcc { lbl: IdentVars, cnd: Cnd },
    #[display(fmt = "andq {src} {dst}")]
    Andq { src: Arg, dst: Arg },
    #[display(fmt = "orq {src} {dst}")]
    Orq { src: Arg, dst: Arg },
    #[display(fmt = "xorq\t{src} {dst}")]
    Xorq { src: Arg, dst: Arg },
    #[display(fmt = "notq\t{dst}")]
    Notq { dst: Arg },
    #[display(fmt = "setcc\t{cnd}")]
    Setcc { cnd: Cnd }, //TODO allow setting other byteregs
    #[display(fmt = "loadlbl\t{lbl} {dst}")]
    LoadLbl { lbl: IdentVars, dst: Arg },
    #[display(fmt = "call_direct\t{lbl} {arity}")]
    CallqDirect { lbl: IdentVars, arity: usize },
    #[display(fmt = "call_indirect\t{src} {arity}")]
    CallqIndirect { src: Arg, arity: usize },
}

#[derive(Debug, PartialEq, Clone, Display, Functor)]
pub enum VarArg<IdentVars: Display> {
    #[display(fmt = "${_0}")]
    Imm(Imm),
    #[display(fmt = "%{_0}")]
    Reg(Reg),
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
    #[display(fmt = "{sym}")]
    XVar { sym: IdentVars },
}

#[derive(Debug, PartialEq, Clone, Display)]
pub enum Imm {
    #[display(fmt = "{_0}")]
    Imm8(u8),
    #[display(fmt = "{_0}")]
    Imm16(u16),
    #[display(fmt = "{_0}")]
    Imm32(u32),
    #[display(fmt = "{_0}")]
    Imm64(u64),
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

/// Refer to [this](https://blog.rchapman.org/posts/Linux_System_Call_Table_for_x86_64/) table and
/// [this](https://man7.org/linux/man-pages/man2/intro.2.html) manual when writing Linux syscalls.
pub const SYSCALL_REGS: [Reg; 7] = [
    Reg::RAX,
    Reg::RDI,
    Reg::RSI,
    Reg::RDX,
    Reg::R10,
    Reg::R8,
    Reg::R9,
];

#[derive(Debug, Hash, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Display)]
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
