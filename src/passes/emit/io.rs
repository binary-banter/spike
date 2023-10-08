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

//We can use: rax rcx rdx rsi rdi r8 r9 r10 r11
fn add_print_block<'p>(blocks: &mut HashMap<&'p str, Block<'p, Arg>>) {
    let mut instrs = vec![
        movq!(reg!(RDI), reg!(RAX)),
        movq!(imm!(10), reg!(RCX)),
        pushq!(imm!(b'\n' as i64))
    ];

    for _ in 0..19 {
        instrs.extend(vec![
            movq!(imm!(0), reg!(RDX)),
            divq!(reg!(RCX)),
            addq!(imm!(b'0' as i64), reg!(RDX)),
            pushq!(reg!(RDX)),
        ])
    }

    for _ in 0..20 {
        instrs.extend(vec![
            movq!(imm!(1), reg!(RAX)), // syscall 1: Write
            movq!(imm!(1), reg!(RDI)), // To stdout
            movq!(reg!(RSP), reg!(RSI)),
            movq!(imm!(1), reg!(RDX)),
            syscall!(),
            addq!(imm!(8), reg!(RSP)),
        ])
    }

    instrs.push(retq!());


    //9_223_372_036_854_775_807 max size
    blocks.insert("_print_int", Block{
        instrs,
    });
}