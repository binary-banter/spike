#[derive(Debug, PartialEq)]
pub struct X86Program {
    pub blocks: Vec<(String, Block<Arg>)>,
    pub stack_space: usize,
}

#[derive(Debug, PartialEq)]
pub struct X86VarProgram {
    pub blocks: Vec<(String, Block<VarArg>)>,
}

#[derive(Debug, PartialEq)]
pub struct Block<A> {
    pub instrs: Vec<Instr<A>>,
}

#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum Instr<A> {
    Instr { cmd: Cmd, args: Vec<A> },
    Callq { lbl: String, arity: usize },
    Retq,
    Jmp { lbl: String },
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Addq,
    Subq,
    Negq,
    Movq,
    Pushq,
    Popq,
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
            Instr::Instr { cmd, args } => Instr::Instr {
                cmd,
                args: args.into_iter().map(From::from).collect(),
            },
            Instr::Callq { lbl, arity } => Instr::Callq { lbl, arity },
            Instr::Retq => Instr::Retq,
            Instr::Jmp { lbl } => Instr::Jmp { lbl },
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

#[macro_export]
macro_rules! block {
    ($($instr:expr),*) => {
        Block { instrs: vec![$($instr),*] }
    };
}

#[macro_export]
macro_rules! instr {
    ($cmd:expr, $($args:expr),*) => {
        Instr::Instr {
            cmd: $cmd,
            args: vec![$($args),*],
        }
    };
}

#[macro_export]
macro_rules! addq {
    ($src:expr, $dst:expr) => {
        Instr::Instr {
            cmd: Cmd::Addq,
            args: vec![$src, $dst],
        }
    };
}

#[macro_export]
macro_rules! subq {
    ($src:expr, $dst:expr) => {
        Instr::Instr {
            cmd: Cmd::Subq,
            args: vec![$src, $dst],
        }
    };
}

#[macro_export]
macro_rules! movq {
    ($src:expr, $dst:expr) => {
        Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![$src, $dst],
        }
    };
}

#[macro_export]
macro_rules! pushq {
    ($src:expr) => {
        Instr::Instr {
            cmd: Cmd::Pushq,
            args: vec![$src],
        }
    };
}

#[macro_export]
macro_rules! popq {
    ($dst:expr) => {
        Instr::Instr {
            cmd: Cmd::Popq,
            args: vec![$dst],
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
        Arg::Imm { val: $val }
    };
}

#[macro_export]
macro_rules! reg {
    ($reg:ident) => {
        Arg::Reg { reg: Reg::$reg }
    };
}
