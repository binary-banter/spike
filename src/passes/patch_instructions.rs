//! This pass compiles `AX86Program`s  into `PX86Program`.
//!
//! This pass makes sure that no instructions use more than one argument that is dereferenced.

use crate::language::x86var::{Arg, Block, Instr, X86Assigned, X86Patched};
use crate::{addq, movq, reg, subq};

impl<'p> X86Assigned<'p> {
    /// See module-level documentation.
    pub fn patch(self) -> X86Patched<'p> {
        X86Patched {
            blocks: self
                .blocks
                .into_iter()
                .map(|(lbl, block)| (lbl, patch_block(block)))
                .collect(),
            entry: self.entry,
            stack_space: self.stack_space,
            std: self.std,
        }
    }
}

fn patch_block(block: Block<'_, Arg>) -> Block<'_, Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .flat_map(patch_instr)
            .collect::<Vec<_>>(),
    }
}

fn patch_instr(instr: Instr<'_, Arg>) -> Vec<Instr<'_, Arg>> {
    match instr {
        Instr::Addq { src, dst } => patch_args(src, dst, |src, dst| addq!(src, dst)),
        Instr::Subq { src, dst } => patch_args(src, dst, |src, dst| subq!(src, dst)),
        Instr::Movq { src, dst } => patch_args(src, dst, |src, dst| movq!(src, dst)),
        _ => vec![instr],
    }
}

fn patch_args<'p>(src: Arg, dst: Arg, op: fn(Arg, Arg) -> Instr<'p, Arg>) -> Vec<Instr<'p, Arg>> {
    match (&src, &dst) {
        (Arg::Deref { .. }, Arg::Deref { .. }) => vec![movq!(src, reg!(RAX)), op(reg!(RAX), dst)],
        _ => vec![op(src, dst)],
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::language::x86var::X86Selected;
    use crate::passes::uniquify::gen_sym;
    use crate::utils::split_test::split_test;
    use crate::{block, callq_direct, movq, reg};
    use test_each_file::test_each_file;

    fn patch_instructions([test]: [&str; 1]) {
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
            .patch()
            .into();

        // Redirect program to exit
        let new_entry = gen_sym("");
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

    test_each_file! { for ["test"] in "./programs/good" as patch_instructions => patch_instructions }
}
