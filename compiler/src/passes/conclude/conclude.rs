use crate::passes::conclude::X86Concluded;
use crate::passes::patch::X86Patched;
use crate::utils::gen_sym::gen_sym;
use crate::*;

impl<'p> X86Patched<'p> {
    #[must_use]
    pub fn conclude(mut self) -> X86Concluded<'p> {
        // let entry = gen_sym("main");
        // self.blocks.insert(
        //     entry,
        //     block!(
        //         pushq!(reg!(RBP)),
        //         movq!(reg!(RSP), reg!(RBP)),
        //         subq!(imm!(self.stack_space as i64), reg!(RSP)),
        //         callq_direct!(self.entry, 0),
        //         movq!(reg!(RAX), reg!(RDI)),
        //         addq!(imm!(self.stack_space as i64), reg!(RSP)),
        //         popq!(reg!(RBP)),
        //         callq_direct!(self.std["exit"], 1)
        //     ),
        // );
        //
        // X86Concluded {
        //     blocks: self.blocks,
        //     entry,
        //     std: self.std,
        // }
        todo!()
    }
}
