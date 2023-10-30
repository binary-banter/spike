use crate::*;
use crate::language::x86var::{X86Concluded, X86Patched};
use crate::utils::gen_sym::gen_sym;

impl<'p> X86Patched<'p> {
    #[must_use]
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
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;
    use crate::passes::select::X86Selected;

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
