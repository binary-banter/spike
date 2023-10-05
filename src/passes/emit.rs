use crate::language::x86var::{Arg, Block, Instr, Reg, SysOp, X86Program};
use std::fmt::{Display, Formatter};
use std::io::Write;

pub fn emit_program(program: X86Program, w: &mut impl Write) -> std::io::Result<()> {
    write!(w, "{program}")
}

impl Display for X86Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "global\t_start")?;
        writeln!(f, "section\t.text")?;
        for (name, block) in &self.blocks {
            write!(f, "{name}:\n{block}")?;
        }
        Ok(())
    }
}

impl Display for Block<Arg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for instr in &self.instrs {
            write!(f, "{instr}")?;
        }
        Ok(())
    }
}

impl Display for Instr<Arg> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instr::Addq { src, dst } => writeln!(f, "\tadd qword {dst}, {src}"),
            Instr::Subq { src, dst } => writeln!(f, "\tsub qword {dst}, {src}"),
            Instr::Negq { dst } => writeln!(f, "\tneg qword {dst}"),
            Instr::Movq { src, dst } => writeln!(f, "\tmov qword {dst}, {src}"),
            Instr::Pushq { src } => writeln!(f, "\tpush qword {src}"),
            Instr::Popq { dst } => writeln!(f, "\tpop qword {dst}"),
            Instr::Callq { lbl, .. } => writeln!(f, "\tcall {lbl}"),
            Instr::Retq => writeln!(f, "\tret"),
            Instr::Jmp { lbl } => writeln!(f, "\tjmp {lbl}"),
            Instr::Syscall { op: SysOp::Exit } => {
                writeln!(f, "mov qword rax, 60")?;
                writeln!(f, "mov qword rdi, 0")?;
                writeln!(f, "syscall")
            }
        }
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Imm { val } => write!(f, "{val}"),
            Arg::Reg { reg } => write!(f, "{reg}"),
            Arg::Deref { reg, off } => {
                if off >= &0 {
                    write!(f, "[{reg}+{off}]")
                } else {
                    write!(f, "[{reg}{off}]")
                }
            }
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reg::RSP => "rsp",
                Reg::RBP => "rbp",
                Reg::RAX => "rax",
                Reg::RBX => "rbx",
                Reg::RCX => "rcx",
                Reg::RDX => "rdx",
                Reg::RSI => "rsi",
                Reg::RDI => "rdi",
                Reg::R8 => "r8",
                Reg::R9 => "r9",
                Reg::R10 => "r10",
                Reg::R11 => "r11",
                Reg::R12 => "r12",
                Reg::R13 => "r13",
                Reg::R14 => "r14",
                Reg::R15 => "r15",
            }
        )
    }
}
