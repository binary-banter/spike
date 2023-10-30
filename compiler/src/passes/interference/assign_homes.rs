use crate::utils::gen_sym::UniqueSym;
use crate::{
    addq, andq, callq_direct, callq_indirect, cmpq, divq, jcc, jmp, load_lbl, movq, mulq, negq,
    notq, orq, popq, pushq, retq, setcc, subq, syscall, xorq,
};
use std::collections::HashMap;
use crate::language::x86var::{Arg, X86Assigned, X86Colored};
use crate::passes::select::{Block, Instr, VarArg};

impl<'p> X86Colored<'p> {
    #[must_use]
    pub fn assign_homes(self) -> X86Assigned<'p> {
        X86Assigned {
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
            entry: self.entry,
            stack_space: self.stack_space,
            std: self.std,
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
        Instr::CallqDirect { lbl, arity } => callq_direct!(lbl, arity),
        Instr::Retq => retq!(),
        Instr::Syscall { arity } => syscall!(arity),
        Instr::Jmp { lbl } => jmp!(lbl),
        Instr::Jcc { lbl, cnd } => jcc!(lbl, cnd),
        Instr::Cmpq { src, dst } => cmpq!(map(src), map(dst)),
        Instr::Andq { src, dst } => andq!(map(src), map(dst)),
        Instr::Orq { src, dst } => orq!(map(src), map(dst)),
        Instr::Xorq { src, dst } => xorq!(map(src), map(dst)),
        Instr::Notq { dst } => notq!(map(dst)),
        Instr::Setcc { cnd } => setcc!(cnd),
        Instr::LoadLbl { sym, dst } => load_lbl!(sym, map(dst)),
        Instr::CallqIndirect { src, arity } => callq_indirect!(map(src), arity),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::gen_sym::gen_sym;
    use crate::utils::split_test::split_test;
    use crate::{block, callq_direct, movq, reg};
    use test_each_file::test_each_file;
    use crate::passes::select::X86Selected;

    fn assign_homes([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let expected_return = expected_return.into();

        let mut program: X86Selected = program
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
            .into();

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

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as assign_homes => assign_homes }
}
