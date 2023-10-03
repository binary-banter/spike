use crate::language::x86var::{Arg, Block, Instr, Reg, VarArg, X86Program, X86VarProgram};
use std::collections::HashMap;

pub fn assign_program(program: X86VarProgram) -> X86Program {
    let mut homes = HashMap::new();

    let blocks = program
        .blocks
        .into_iter()
        .map(|block| (block.0, assign_block(block.1, &mut homes)))
        .collect();

    X86Program {
        blocks,
        stack_space: homes.len().div_ceil(16) * 16,
    }
}

fn assign_block(block: Block<VarArg>, homes: &mut HashMap<String, i64>) -> Block<Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .map(|instr| assign_instruction(instr, homes))
            .collect(),
    }
}

fn assign_instruction(instr: Instr<VarArg>, homes: &mut HashMap<String, i64>) -> Instr<Arg> {
    match instr {
        Instr::Instr { cmd, args } => Instr::Instr {
            cmd,
            args: args
                .into_iter()
                .map(|arg| match arg {
                    VarArg::Imm { val } => Arg::Imm { val },
                    VarArg::Reg { reg } => Arg::Reg { reg },
                    VarArg::Deref { reg, off } => Arg::Deref { reg, off },
                    VarArg::XVar { sym } => {
                        if !homes.contains_key(&sym) {
                            homes.insert(sym.clone(), -(homes.len() as i64 + 1) * 8);
                        }
                        Arg::Deref {
                            reg: Reg::RBP,
                            off: homes[&sym],
                        }
                    }
                })
                .collect(),
        },
        Instr::Callq { lbl, arity } => Instr::Callq { lbl, arity },
        Instr::Retq => Instr::Retq,
        Instr::Jmp { lbl } => Instr::Jmp { lbl },
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::passes::assign_homes::assign_program;
    use crate::passes::explicate_control::explicate_program;
    use crate::passes::remove_complex_operands::rco_program;
    use crate::passes::select_instructions::select_program;
    use crate::passes::uniquify::uniquify_program;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn assign_homes([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = assign_program(select_program(explicate_program(rco_program(
            uniquify_program(program),
        ))));
        let mut io = TestIO::new(input);
        let result = interpret_x86var("start", &program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as assign_homes => assign_homes }
}
