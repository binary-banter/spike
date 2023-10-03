pub type X86Program = GenericX86Program<Arg>;
pub type X86VarProgram = GenericX86Program<VarArg>;

#[derive(Debug, PartialEq)]
pub struct GenericX86Program<A> {
    pub blocks: Vec<(String, Block<A>)>,
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
