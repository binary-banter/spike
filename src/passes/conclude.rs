use crate::language::x86var::{Arg, Block, Instr, PX86Program, Reg, X86Program};
use crate::{addq, block, callq, imm, jmp, movq, popq, pushq, reg, subq};

impl<'p> PX86Program<'p> {
    pub fn conclude(mut self) -> X86Program<'p> {
        main(&mut self);
        core(&mut self);
        conclusion(&mut self);

        X86Program {
            blocks: self.blocks,
        }
    }
}

fn core(program: &mut PX86Program) {
    program
        .blocks
        .get_mut("core")
        .expect("There should be a core block.")
        .instrs
        .extend([jmp!("conclusion")]);
}

fn main(program: &mut PX86Program) {
    program.blocks.insert(
        "main",
        block!(
            pushq!(reg!(RBP)),
            movq!(reg!(RSP), reg!(RBP)),
            subq!(imm!(program.stack_space as i64), reg!(RSP)),
            jmp!("core")
        ),
    );
}

fn conclusion(program: &mut PX86Program) {
    program.blocks.insert(
        "conclusion",
        block!(
            addq!(imm!(program.stack_space as i64), reg!(RSP)),
            popq!(reg!(RBP)),
            movq!(reg!(RAX), reg!(RDI)),
            callq!("exit", 1)
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn conclude([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select()
            .assign_homes()
            .patch()
            .conclude();

        let mut io = TestIO::new(input);
        let result = interpret_x86var("main", &program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as conclude => conclude }
}
