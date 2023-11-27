pub mod interpreter;
pub mod macros;
pub mod select;
pub mod std_lib;
#[cfg(test)]
mod tests;

use crate::passes::select::std_lib::Std;
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Display)]
#[display(
    fmt = "{}",
    r#"blocks.iter().map(|(sym, block)| format!("{sym}:\n{block}")).format("\n")"#
)]
pub struct X86Selected<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug, Clone, Display, Functor)]
#[display(fmt = "\t{}", r#"instrs.iter().format("\n\t")"#)]
pub struct Block<'p, A: Display> {
    pub instrs: Vec<Instr<A, UniqueSym<'p>>>,
}

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
    #[display(fmt = "addq\t{src}\t{dst}")]
    Addq { src: Arg, dst: Arg },
    #[display(fmt = "subq\t{src}\t{dst}")]
    Subq { src: Arg, dst: Arg },
    #[display(fmt = "divq\t{divisor}")]
    Divq { divisor: Arg },
    #[display(fmt = "mulq\t{src}")]
    Mulq { src: Arg },
    #[display(fmt = "negq\t{dst}")]
    Negq { dst: Arg },
    #[display(fmt = "movq\t{src}\t{dst}")]
    Movq { src: Arg, dst: Arg },
    #[display(fmt = "pushq\t{src}")]
    Pushq { src: Arg },
    #[display(fmt = "popq\t{dst}")]
    Popq { dst: Arg },
    #[display(fmt = "retq")]
    Retq,
    #[display(fmt = "syscall\t// arity: {arity}")]
    Syscall { arity: usize },
    #[display(fmt = "cmpq\t{src}\t{dst}")]
    Cmpq { src: Arg, dst: Arg },
    #[display(fmt = "jmp\t{lbl}")]
    Jmp { lbl: IdentVars },
    #[display(fmt = "jcc\t{cnd}\t{lbl}")]
    Jcc { lbl: IdentVars, cnd: Cnd },
    #[display(fmt = "andq {src}\t{dst}")]
    Andq { src: Arg, dst: Arg },
    #[display(fmt = "orq {src}\t{dst}")]
    Orq { src: Arg, dst: Arg },
    #[display(fmt = "xorq\t{src}\t{dst}")]
    Xorq { src: Arg, dst: Arg },
    #[display(fmt = "notq\t{dst}")]
    Notq { dst: Arg },
    #[display(fmt = "setcc\t{cnd}")]
    Setcc { cnd: Cnd }, //TODO allow setting other byteregs
    #[display(fmt = "loadlbl\t{sym}\t{dst}")]
    LoadLbl { sym: IdentVars, dst: Arg },
    #[display(fmt = "call_direct\t{lbl}\t// arity: {arity}")]
    CallqDirect { lbl: IdentVars, arity: usize },
    #[display(fmt = "call_indirect\t{src}\t// arity: {arity}")]
    CallqIndirect { src: Arg, arity: usize },
}

#[derive(Debug, PartialEq, Clone, Display, Functor)]
pub enum VarArg<IdentVars: Display> {
    #[display(fmt = "${val}")]
    Imm { val: i64 },
    #[display(fmt = "%{reg}")]
    Reg { reg: Reg },
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
    #[display(fmt = "{sym}")]
    XVar { sym: IdentVars },
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

pub const SYSCALL_REGS: [Reg; 7] = [
    Reg::RAX,
    Reg::RDI,
    Reg::RSI,
    Reg::RDX,
    Reg::RCX,
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
