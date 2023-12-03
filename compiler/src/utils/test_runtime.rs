use crate::passes::select::{FunSelected, X86Selected};
use crate::utils::gen_sym::gen_sym;
use crate::*;
use std::collections::HashMap;

/// Adds runtime for testing.
pub fn add_runtime(program: &mut X86Selected) {
    let runtime = gen_sym("runtime");
    let entry = gen_sym("entry");
    let exit = gen_sym("exit");

    let entry_block = block!(callq_direct!(program.entry, 0), jmp!(exit));
    let exit_block = block!(
        movq!(reg!(RAX), reg!(RDI)),
        movq!(imm!(0x3C), reg!(RAX)),
        syscall!(2)
    );

    let runtime_fn = FunSelected {
        blocks: HashMap::from([(entry, entry_block), (exit, exit_block)]),
        entry,
        exit,
    };

    program.fns.insert(runtime, runtime_fn);
    program.entry = runtime;
}
