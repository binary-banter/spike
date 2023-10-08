use std::collections::HashMap;
use crate::*;
use crate::passes::emit::Instr;
use crate::passes::emit::Reg;
use crate::language::x86var::{Arg, Block};

pub fn add_io_blocks<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    add_exit_block(blocks);
    add_print_block(blocks);
}

fn add_exit_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    blocks.insert("exit", block!(
        movq!(imm!(0x3C), reg!(RAX)),
        syscall!()
    ));
}

fn add_print_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    /* print block
mov rax 1 // syscall 1
mov rdi 1 // STDOUT
mov rsi, msg
mov rdx, msglen
syscall
 */
}