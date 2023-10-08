use std::collections::HashMap;
use crate::*;
use crate::passes::emit::Instr;
use crate::passes::emit::Reg;
use crate::language::x86var::{Arg, Block};
use crate::language::x86var::Cnd;

pub fn add_io_blocks<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    add_exit_block(blocks);
    add_print_block(blocks);
    add_read_block(blocks);
}

fn add_exit_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    blocks.insert("exit", block!(
        movq!(imm!(0x3C), reg!(RAX)),
        syscall!()
    ));
}

//We can use: rax rcx rdx rsi rdi r8 r9 r10 r11
fn add_print_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    blocks.insert("_print_int", block!(
        movq!(imm!(10), reg!(RCX)),
        pushq!(imm!(b'\n' as i64)),
        movq!(reg!(RDI), reg!(RAX)),

        addq!(imm!(0), reg!(RAX)),
        movq!(imm!(0), reg!(RSI)),

        jcc!("_print_int_neg", Cnd::Sign),
        jmp!("_print_int_push_loop")
    ));
    blocks.insert("_print_int_neg", block!(
        movq!(imm!(1), reg!(RSI)),
        negq!(reg!(RAX)),
        jmp!("_print_int_push_loop")
    ));
    blocks.insert("_print_int_push_loop", block!(
        movq!(imm!(0), reg!(RDX)),
        divq!(reg!(RCX)),
        addq!(imm!(b'0' as i64), reg!(RDX)),
        pushq!(reg!(RDX)),

        addq!(imm!(0), reg!(RAX)),
        jcc!("_print_int_push_loop", Cnd::NotEqual),

        addq!(imm!(0), reg!(RSI)),
        jcc!("_print_int_print_loop", Cnd::Equal),
        pushq!(imm!(b'-' as i64)),

        jmp!("_print_int_print_loop")
    ));
    blocks.insert("_print_int_print_loop", block!(
        // Print top of stack
        movq!(imm!(1), reg!(RAX)), // syscall 1: Write
        movq!(imm!(1), reg!(RDI)), // STDOUT
        movq!(reg!(RSP), reg!(RSI)),
        movq!(imm!(1), reg!(RDX)),
        syscall!(),

        // Check if we continue
        popq!(reg!(RAX)),
        subq!(imm!(b'\n' as i64), reg!(RAX)),
        jcc!("_print_int_print_loop", Cnd::NotEqual),
        jmp!("_print_int_exit")
    ));
    blocks.insert("_print_int_exit", block!(
        retq!()
    ));
}

fn add_read_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    blocks.insert("_read_int", block!(
        pushq!(reg!(RBX)),              // save a callee-saved register
        movq!(imm!(0), reg!(RBX)),      // zero out RBX
        subq!(imm!(8), reg!(RSP)),      // allocate some space on the stack for reading the next byte
        jmp!("_read_int_loop")
    ));

    blocks.insert("_read_int_loop", block!(
        movq!(imm!(0), reg!(RAX)),      // READ = 0
        movq!(imm!(0), reg!(RDI)),      // STDIN = 0
        movq!(reg!(RSP), reg!(RSI)),    // RSI is pointer to allocated byte
        movq!(imm!(1), reg!(RDX)),      // bytes to read = 1
        syscall!(),

        movq!(deref!(RSP, 0), reg!(RAX)),

        // check if newline
        movq!(reg!(RAX), reg!(RCX)),
        subq!(imm!(b'\n' as i64), reg!(RCX)),
        jcc!("_read_int_exit", Cnd::Equal),

        movq!(imm!(66), reg!(RDI)),
        // check if >b'9'
        movq!(reg!(RAX), reg!(RCX)),
        subq!(imm!(b'9' as i64), reg!(RCX)),
        jcc!("exit", Cnd::Greater),

        // check if <b'0'
        movq!(reg!(RAX), reg!(RCX)),
        subq!(imm!(b'0' as i64), reg!(RCX)),
        jcc!("exit", Cnd::Less),

        movq!(imm!(10), reg!(RAX)),
        mulq!(reg!(RBX)),
        movq!(reg!(RAX), reg!(RBX)),

        movq!(deref!(RSP, 0), reg!(RAX)),
        subq!(imm!(b'0' as i64), reg!(RAX)),
        addq!(reg!(RAX), reg!(RBX)),

        jmp!("_read_int_loop")
    ));

    blocks.insert("_read_int_exit", block!(
        movq!(reg!(RBX), reg!(RAX)),
        addq!(imm!(8), reg!(RSP)),
        popq!(reg!(RBX)),
        retq!()
    ));
}