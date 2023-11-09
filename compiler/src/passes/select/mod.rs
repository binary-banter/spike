use crate::passes::select::io::Std;
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::Display;

pub mod interpreter;
pub mod io;
pub mod macros;
pub mod select;

#[derive(Debug, PartialEq, Display)]
#[display(
    fmt = "{}",
    r#"blocks.iter().map(|(sym, block)| format!("{sym}:\n{block}")).format("\n")"#
)]
pub struct X86Selected<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq, Clone, Display, Functor)]
#[display(fmt = "\t{}", r#"instrs.iter().format("\n\t")"#)]
pub struct Block<'p, A: Display> {
    pub instrs: Vec<Instr<'p, A>>,
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
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

#[derive(Clone, Debug, PartialEq, Display, Functor)]
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

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Display)]
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

#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash, Ord, PartialOrd, Display)]
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

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::passes::parse::parse::parse_program;
    use crate::utils::gen_sym::gen_sym;
    use crate::utils::split_test::split_test;
    use crate::{block, callq_direct, movq, reg};
    use test_each_file::test_each_file;

    fn select([test]: [&str; 1]) {
        let (input, expected_output, expected_return, _) = split_test(test);

        let mut program = parse_program(test)
            .unwrap()
            .validate()
            .unwrap()
            .reveal()
            .atomize()
            .explicate()
            .eliminate()
            .select();

        // Redirect program to exit
        let new_entry = gen_sym("tmp");
        program.blocks.insert(
            new_entry,
            block!(
                callq_direct!(program.entry, 0),
                movq!(reg!(RAX), reg!(RDI)),
                callq_direct!(program.std.exit, 1)
            ),
        );
        program.entry = new_entry;

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as select_instructions => select }
}
