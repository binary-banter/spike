use crate::passes::assign::{Arg, X86Assigned};
use crate::passes::patch::X86Patched;
use crate::passes::select::{Block, Instr};
use crate::{addq, movq, reg, subq};
use crate::utils::gen_sym::UniqueSym;

impl<'p> X86Assigned<'p> {
    #[must_use]
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

fn patch_block<'p>(block: Block<'p, Arg>) -> Block<'p, Arg> {
    Block {
        instrs: block
            .instrs
            .into_iter()
            .flat_map(patch_instr)
            .collect::<Vec<_>>(),
    }
}

fn patch_instr<'p>(instr: Instr<Arg, UniqueSym<'p>, >) -> Vec<Instr<Arg, UniqueSym<'p>, >> {
    match instr {
        Instr::Addq { src, dst } => patch_args(src, dst, |src, dst| addq!(src, dst)),
        Instr::Subq { src, dst } => patch_args(src, dst, |src, dst| subq!(src, dst)),
        Instr::Movq { src, dst } => patch_args(src, dst, |src, dst| movq!(src, dst)),
        _ => vec![instr],
    }
}

fn patch_args<'p>(src: Arg, dst: Arg, op: fn(Arg, Arg) -> Instr<Arg, UniqueSym<'p>>) -> Vec<Instr<Arg, UniqueSym<'p>>> {
    match (&src, &dst) {
        (Arg::Deref { .. }, Arg::Deref { .. }) => vec![movq!(src, reg!(RAX)), op(reg!(RAX), dst)],
        _ => vec![op(src, dst)],
    }
}
