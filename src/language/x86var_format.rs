use std::fmt::{Display, Formatter};
use crate::language::x86var::{Block, Cnd, Instr, Reg, VarArg, X86Selected};
use crate::passes::uniquify::UniqueSym;

impl Display for X86Selected<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, block) in &self.blocks {
            writeln!(f, "{sym}:")?;
            write!(f, "{block}")?;
        }
        Ok(())
    }
}

impl<A: Display> Display for Block<'_, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for instr in &self.instrs {
            writeln!(f, "\t{instr}")?;
        }
        Ok(())
    }
}

impl<A: Display> Display for Instr<'_, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instr::Addq { src, dst } => write!(f, "addq\t{src}\t{dst}"),
            Instr::Subq { src, dst } => write!(f, "subq\t{src}\t{dst}"),
            Instr::Divq { divisor } => write!(f, "divq\t{divisor}"),
            Instr::Mulq { src } => write!(f, "mulq\t{src}"),
            Instr::Negq { dst } => write!(f, "negq\t{dst}"),
            Instr::Movq { src, dst } => write!(f, "movq\t{src}\t{dst}"),
            Instr::Pushq { src } => write!(f, "pushq\t{src}"),
            Instr::Popq { dst } => write!(f, "popq\t{dst}"),
            Instr::Retq => write!(f, "retq"),
            Instr::Syscall { arity } => write!(f, "syscall\t// arity: {arity}"),
            Instr::Cmpq { src, dst } => write!(f, "cmpq\t{src}\t{dst}"),
            Instr::Jmp { lbl } => write!(f, "jmp\t{lbl}"),
            Instr::Jcc { lbl, cnd } => write!(f, "jcc\t{cnd}\t{lbl}"),
            Instr::Andq { src, dst } => write!(f, "andq {src}\t{dst}"),
            Instr::Orq { src, dst } => write!(f, "orq {src}\t{dst}"),
            Instr::Xorq { src, dst } => write!(f, "xorq\t{src}\t{dst}"),
            Instr::Notq { dst } => write!(f, "notq\t{dst}"),
            Instr::Setcc { cnd } => write!(f, "setcc\t{cnd}"),
            Instr::LoadLbl { sym, dst } => write!(f, "loadlbl\t{sym}\t{dst}"),
            Instr::CallqDirect { lbl, arity } => write!(f, "call_direct\t{lbl}\t// arity: {arity}"),
            Instr::CallqIndirect { src, arity } => write!(f, "call_indirect\t{src}\t// arity: {arity}"),
        }
    }
}

impl Display for Cnd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for UniqueSym<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.sym, self.id)
    }
}

impl Display for VarArg<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VarArg::Imm { val } => write!(f, "${val}"),
            VarArg::Reg { reg } => write!(f, "%{reg}"),
            VarArg::Deref { reg, off } => write!(f, "[%{reg} + ${off}]"),
            VarArg::XVar { sym } => write!(f, "'{sym}'"),
        }
    }
}