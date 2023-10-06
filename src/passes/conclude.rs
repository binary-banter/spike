use crate::language::x86var::{Arg, Block, Instr, PX86Program, Reg, SysOp, X86Program};
use crate::{addq, block, callq, imm, jmp, movq, popq, pushq, reg, subq};

pub fn conclude_program(mut program: PX86Program) -> X86Program {
    main(&mut program);
    core(&mut program);
    conclusion(&mut program);

    let program = X86Program {
        blocks: program.blocks,
    };

    program
}

fn core(program: &mut PX86Program) {
    program
        .blocks
        .get_mut("core")
        .expect("There should be a start block.")
        .instrs
        .extend([jmp!("conclusion")]);
}

fn main(program: &mut PX86Program) {
    program.blocks.insert(
        "main".to_string(),
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
        "conclusion".to_string(),
        block!(
            addq!(imm!(program.stack_space as i64), reg!(RSP)),
            popq!(reg!(RBP)),
            movq!(imm!(0), reg!(RDI)),
            callq!("exit", 1)
        ),
    );
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::passes::assign_homes::assign_program;
    use crate::passes::conclude::conclude_program;
    use crate::passes::explicate_control::explicate_program;
    use crate::passes::patch_instructions::patch_program;
    use crate::passes::remove_complex_operands::rco_program;
    use crate::passes::select_instructions::select_program;
    use crate::passes::uniquify::uniquify_program;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn conclude([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = conclude_program(patch_program(assign_program(select_program(
            explicate_program(rco_program(uniquify_program(program))),
        ))));

        let mut io = TestIO::new(input);
        let result = interpret_x86var("main", &program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as conclude => conclude }
}