//! This pass compiles `X86VarProgram`s  into `AX86Program`.
//!
//! This pass is responsible for assigning all the program variables to locations on the stack.

use crate::language::x86var::{AX86Program, Arg, Block, CX86VarProgram, Instr, VarArg};
use crate::passes::uniquify::UniqueSym;
use crate::{addq, callq, divq, jcc, jmp, movq, mulq, negq, popq, pushq, retq, subq, syscall};
use std::collections::HashMap;

impl<'p> CX86VarProgram<'p> {
    //! See module-level documentation.
    pub fn assign_homes(self) -> AX86Program<'p> {
        AX86Program {
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| {
                    (
                        name,
                        Block {
                            instrs: block
                                .instrs
                                .into_iter()
                                .map(|instr| assign_instr(instr, &self.color_map))
                                .collect(),
                        },
                    )
                })
                .collect(),
            stack_space: self.stack_space,
        }
    }
}

fn assign_instr<'p>(
    instr: Instr<'p, VarArg>,
    color_map: &HashMap<UniqueSym, Arg>,
) -> Instr<'p, Arg> {
    let map = |arg: VarArg| -> Arg {
        match arg {
            VarArg::Imm { val } => Arg::Imm { val },
            VarArg::Reg { reg } => Arg::Reg { reg },
            VarArg::Deref { reg, off } => Arg::Deref { reg, off },
            VarArg::XVar { sym } => color_map[&sym],
        }
    };

    match instr {
        Instr::Addq { src, dst } => addq!(map(src), map(dst)),
        Instr::Subq { src, dst } => subq!(map(src), map(dst)),
        Instr::Divq { divisor } => divq!(map(divisor)),
        Instr::Mulq { src } => mulq!(map(src)),
        Instr::Negq { dst } => negq!(map(dst)),
        Instr::Movq { src, dst } => movq!(map(src), map(dst)),
        Instr::Pushq { src } => pushq!(map(src)),
        Instr::Popq { dst } => popq!(map(dst)),
        Instr::Callq { lbl, arity } => callq!(lbl, arity),
        Instr::Retq => retq!(),
        Instr::Syscall => syscall!(),
        Instr::Jmp { lbl } => jmp!(lbl),
        Instr::Jcc { lbl, cnd } => jcc!(lbl, cnd),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::language::x86var::X86VarProgram;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn assign_homes([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: X86VarProgram = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select()
            .add_liveness()
            .compute_interference()
            .color_interference()
            .assign_homes()
            .into();
        let mut io = TestIO::new(input);
        let result = program.interpret("core", &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as assign_homes => assign_homes }
}
