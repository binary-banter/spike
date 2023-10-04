use crate::language::x86var::{Arg, Block, Cmd, Instr, Reg, X86Program};

pub fn conclude_program(mut program: X86Program) -> X86Program {
    start(&mut program);
    main(&mut program);
    conclusion(&mut program);

    program
}

fn start(program: &mut X86Program) {
    program.blocks[0].1.instrs.extend([Instr::Jmp {
        lbl: "conclusion".to_string(),
    }]);
}

fn main(program: &mut X86Program) {
    // pushq %rbp
    // movq %rsp %rbp
    // subq $stackSpace %rsp
    // jmp start
    program.blocks.push((
        "main".to_string(),
        Block {
            instrs: vec![
                Instr::Instr {
                    cmd: Cmd::Pushq,
                    args: vec![Arg::Reg { reg: Reg::RBP }],
                },
                Instr::Instr {
                    cmd: Cmd::Movq,
                    args: vec![Arg::Reg { reg: Reg::RSP }, Arg::Reg { reg: Reg::RBP }],
                },
                Instr::Instr {
                    cmd: Cmd::Subq,
                    args: vec![
                        Arg::Imm {
                            val: program.stack_space as i64,
                        },
                        Arg::Reg { reg: Reg::RSP },
                    ],
                },
                Instr::Jmp {
                    lbl: "start".to_string(),
                },
            ],
        },
    ));
}

fn conclusion(program: &mut X86Program) {
    // addq $stackSpace %rsp
    // popq %rbp
    // retq
    program.blocks.push((
        "conclusion".to_string(),
        Block {
            instrs: vec![
                Instr::Instr {
                    cmd: Cmd::Addq,
                    args: vec![
                        Arg::Imm {
                            val: program.stack_space as i64,
                        },
                        Arg::Reg { reg: Reg::RSP },
                    ],
                },
                Instr::Instr {
                    cmd: Cmd::Popq,
                    args: vec![Arg::Reg { reg: Reg::RBP }],
                },
                Instr::Retq,
            ],
        },
    ));
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
