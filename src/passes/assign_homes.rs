use crate::language::x86var::{AX86Program, Arg, Block, Instr, Reg, VarArg, X86VarProgram};
use crate::{addq, callq, jmp, movq, negq, popq, pushq, retq, subq};
use std::collections::HashMap;
use crate::passes::uniquify::UniqueSym;

impl<'p> X86VarProgram<'p> {
    pub fn assign_homes(self) -> AX86Program<'p> {
        let mut homes = HashMap::new();

        AX86Program {
            blocks: self
                .blocks
                .into_iter()
                .map(|block| (block.0, assign_block(block.1, &mut homes)))
                .collect(),
            stack_space: (8 * homes.len()).div_ceil(16) * 16,
        }
    }
}

fn assign_block<'p>(block: Block<'p, VarArg<'p>>, homes: &mut HashMap<UniqueSym<'p>, i64>) -> Block<'p, Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .map(|instr| assign_instruction(instr, homes))
            .collect(),
    }
}

fn assign_instruction<'p>(instr: Instr<'p, VarArg<'p>>, homes: &mut HashMap<UniqueSym<'p>, i64>) -> Instr<'p, Arg> {
    match instr {
        Instr::Addq { src, dst } => addq!(assign_arg(src, homes), assign_arg(dst, homes)),
        Instr::Subq { src, dst } => subq!(assign_arg(src, homes), assign_arg(dst, homes)),
        Instr::Negq { dst } => negq!(assign_arg(dst, homes)),
        Instr::Movq { src, dst } => movq!(assign_arg(src, homes), assign_arg(dst, homes)),
        Instr::Pushq { src } => pushq!(assign_arg(src, homes)),
        Instr::Popq { dst } => popq!(assign_arg(dst, homes)),
        Instr::Callq { lbl, arity } => callq!(lbl, arity),
        Instr::Retq => retq!(),
        Instr::Jmp { lbl } => jmp!(lbl),
    }
}

fn assign_arg<'p>(arg: VarArg<'p>, homes: &mut HashMap<UniqueSym<'p>, i64>) -> Arg {
    match arg {
        VarArg::Imm { val } => Arg::Imm { val },
        VarArg::Reg { reg } => Arg::Reg { reg },
        VarArg::Deref { reg, off } => Arg::Deref { reg, off },
        VarArg::XVar { sym } => {
            if !homes.contains_key(&sym) {
                homes.insert(sym, -(homes.len() as i64 + 1) * 8);
            }
            Arg::Deref {
                reg: Reg::RBP,
                off: homes[&sym],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn assign_homes([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select()
            .assign_homes();
        let mut io = TestIO::new(input);
        let result = interpret_x86var("core", &program.into(), &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as assign_homes => assign_homes }
}
