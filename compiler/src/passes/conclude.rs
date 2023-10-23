//! This pass compiles `PX86Program`s  into `X86Program`.
//!
//! This pass generates the entry and exit point for the program wrapped around the body of the `PX86Program` program.
//! Note that we will refer to the body of the `PX86Program` program as the 'core' block.

use crate::language::x86var::{X86Concluded, X86Patched};
use crate::passes::uniquify::gen_sym;
use crate::{addq, block, callq_direct, imm, movq, popq, pushq, reg, subq};

impl<'p> X86Patched<'p> {
    /// See module-level documentation.
    pub fn conclude(mut self) -> X86Concluded<'p> {
        let entry = gen_sym("main");
        self.blocks.insert(
            entry,
            block!(
                pushq!(reg!(RBP)),
                movq!(reg!(RSP), reg!(RBP)),
                subq!(imm!(self.stack_space as i64), reg!(RSP)),
                callq_direct!(self.entry, 0),
                movq!(reg!(RAX), reg!(RDI)),
                addq!(imm!(self.stack_space as i64), reg!(RSP)),
                popq!(reg!(RBP)),
                callq_direct!(self.std.exit, 1)
            ),
        );

        X86Concluded {
            blocks: self.blocks,
            entry,
            std: self.std,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::language::x86var::X86Selected;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn conclude([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let expected_return = expected_return.into();

        let program: X86Selected = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .explicate()
            .select()
            .add_liveness()
            .compute_interference()
            .color_interference()
            .assign_homes()
            .patch()
            .conclude()
            .into();
        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as conclude => conclude }
}
