use crate::language::x86var::{Arg, Block, Cmd, Instr, Reg, X86Program};
use crate::{instr, movq, reg};

pub fn patch_program(program: X86Program) -> X86Program {
    X86Program {
        blocks: program
            .blocks
            .into_iter()
            .map(|(lbl, block)| (lbl, patch_block(block)))
            .collect(),
        stack_space: program.stack_space,
    }
}

pub fn patch_block(block: Block<Arg>) -> Block<Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .flat_map(patch_instr)
            .collect::<Vec<_>>(),
    }
}

pub fn patch_instr(instr: Instr<Arg>) -> Vec<Instr<Arg>> {
    match instr {
        Instr::Instr { cmd, args } => match args.as_slice() {
            [a1 @ Arg::Deref { .. }, a2 @ Arg::Deref { .. }] => {
                vec![
                    movq!(a1.clone(), reg!(RAX)),
                    instr!(cmd, reg!(RAX), a2.clone()),
                ]
            }
            _ => vec![Instr::Instr { cmd, args }],
        },
        _ => vec![instr],
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::passes::assign_homes::assign_program;
    use crate::passes::explicate_control::explicate_program;
    use crate::passes::patch_instructions::patch_program;
    use crate::passes::remove_complex_operands::rco_program;
    use crate::passes::select_instructions::select_program;
    use crate::passes::uniquify::uniquify_program;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn patch_instructions([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = patch_program(assign_program(select_program(explicate_program(
            rco_program(uniquify_program(program)),
        ))));
        let mut io = TestIO::new(input);
        let result = interpret_x86var("start", &program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as patch_instructions => patch_instructions }
}
