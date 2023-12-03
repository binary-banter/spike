use functor_derive::Functor;
use crate::passes::conclude::X86Concluded;
use crate::passes::patch::X86Patched;
use crate::utils::gen_sym::gen_sym;
use crate::*;
use crate::passes::select::CALLEE_SAVED_NO_STACK;

impl<'p> X86Patched<'p> {
    #[must_use]
    pub fn conclude(mut self) -> X86Concluded<'p> {
        // let blocks = self.fns.into_iter().flat_map(|(_, mut fun)| {
        //     // fun.blocks.get_mut(&fun.entry).unwrap().instrs.(2 + CALLEE_SAVED_NO_STACK.len(), )
        //
        //     fun.blocks.into_iter().map(|(block_sym, block)| {
        //         // let block = match block_sym {
        //         //     s if s == fun.entry => ,
        //         //     s if s == fun.exit => todo!(),
        //         //     _ => block,
        //         // };
        //
        //         (block_sym, block)
        //     })
        // }).collect();

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

        todo!()
        // X86Concluded {
        //     blocks,
        //     entry,
        // }
    }
}
