pub mod interpreter;
pub mod std_lib;
pub mod macros;
pub mod select;
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
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Clone, Display, Functor)]
#[display(fmt = "\t{}", r#"instrs.iter().format("\n\t")"#)]
pub struct Block<'p, A: Display> {
    pub instrs: Vec<Instr<'p, A>>,
}

#[derive(Copy, Clone, PartialEq, Display)]
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

#[derive(Clone, PartialEq, Display, Functor)]
pub enum Instr<'p, A: Display> {
    #[display(fmt = "addq\t{src}\t{dst}")]
    Addq { src: A, dst: A },
    #[display(fmt = "subq\t{src}\t{dst}")]
    Subq { src: A, dst: A },
    #[display(fmt = "divq\t{divisor}")]
    Divq { divisor: A },
    #[display(fmt = "mulq\t{src}")]
    Mulq { src: A },
    #[display(fmt = "negq\t{dst}")]
    Negq { dst: A },
    #[display(fmt = "movq\t{src}\t{dst}")]
    Movq { src: A, dst: A },
    #[display(fmt = "pushq\t{src}")]
    Pushq { src: A },
    #[display(fmt = "popq\t{dst}")]
    Popq { dst: A },
    #[display(fmt = "retq")]
    Retq,
    #[display(fmt = "syscall\t// arity: {arity}")]
    Syscall { arity: usize },
    #[display(fmt = "cmpq\t{src}\t{dst}")]
    Cmpq { src: A, dst: A },
    #[display(fmt = "jmp\t{lbl}")]
    Jmp { lbl: UniqueSym<'p> },
    #[display(fmt = "jcc\t{cnd}\t{lbl}")]
    Jcc { lbl: UniqueSym<'p>, cnd: Cnd },
    #[display(fmt = "andq {src}\t{dst}")]
    Andq { src: A, dst: A },
    #[display(fmt = "orq {src}\t{dst}")]
    Orq { src: A, dst: A },
    #[display(fmt = "xorq\t{src}\t{dst}")]
    Xorq { src: A, dst: A },
    #[display(fmt = "notq\t{dst}")]
    Notq { dst: A },
    #[display(fmt = "setcc\t{cnd}")]
    Setcc { cnd: Cnd }, //TODO allow setting other byteregs
    #[display(fmt = "loadlbl\t{sym}\t{dst}")]
    LoadLbl { sym: UniqueSym<'p>, dst: A },
    #[display(fmt = "call_direct\t{lbl}\t// arity: {arity}")]
    CallqDirect { lbl: UniqueSym<'p>, arity: usize },
    #[display(fmt = "call_indirect\t{src}\t// arity: {arity}")]
    CallqIndirect { src: A, arity: usize },
}

#[derive(PartialEq, Clone, Display)]
pub enum VarArg<'p> {
    #[display(fmt = "${val}")]
    Imm { val: i64 },
    #[display(fmt = "%{reg}")]
    Reg { reg: Reg },
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
    #[display(fmt = "{sym}")]
    XVar { sym: UniqueSym<'p> },
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

#[derive(Hash, Clone, Copy, Eq, Ord, PartialEq, PartialOrd, Display)]
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
