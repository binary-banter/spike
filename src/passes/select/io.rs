use crate::language::x86var::Reg;
use crate::language::x86var::{Block, Cnd, VarArg};
use crate::passes::uniquify::{gen_sym, UniqueSym};
use crate::{
    addq, block, cmpq, deref, divq, imm, jcc, jmp, movq, mulq, negq, popq, pushq, reg, retq, subq,
    syscall,
};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Std<'p> {
    pub exit: UniqueSym<'p>,
    pub print_int: UniqueSym<'p>,
    pub read_int: UniqueSym<'p>,
}

impl<'p> Std<'p> {
    pub fn new(blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>) -> Self {
        let exit = add_exit_block(blocks);

        Std {
            exit,
            print_int: add_print_block(blocks),
            read_int: add_read_block(blocks, exit),
        }
    }
}

fn add_exit_block<'p>(blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>) -> UniqueSym<'p> {
    let entry = gen_sym("exit");
    blocks.insert(entry, block!(movq!(imm!(0x3C), reg!(RAX)), syscall!()));
    entry
}

//We can use: rax rcx rdx rsi rdi r8 r9 r10 r11
fn add_print_block<'p>(blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>) -> UniqueSym<'p> {
    let entry = gen_sym("print_int");
    let print_int_neg = gen_sym("print_int_neg");
    let print_int_push_loop = gen_sym("print_int_push_loop");
    let print_int_print_loop = gen_sym("print_int_print_loop");
    let print_int_exit = gen_sym("print_int_exit");

    blocks.insert(
        entry,
        block!(
            movq!(imm!(10), reg!(RCX)),
            pushq!(imm!(i64::from(b'\n'))),
            movq!(reg!(RDI), reg!(RAX)),
            movq!(imm!(0), reg!(RSI)),
            cmpq!(imm!(0), reg!(RAX)),
            jcc!(print_int_neg, Cnd::Sign),
            jmp!(print_int_push_loop)
        ),
    );
    blocks.insert(
        print_int_neg,
        block!(
            movq!(imm!(1), reg!(RSI)),
            negq!(reg!(RAX)),
            jmp!(print_int_push_loop)
        ),
    );
    blocks.insert(
        print_int_push_loop,
        block!(
            movq!(imm!(0), reg!(RDX)),
            divq!(reg!(RCX)),
            addq!(imm!(i64::from(b'0')), reg!(RDX)),
            pushq!(reg!(RDX)),
            cmpq!(imm!(0), reg!(RAX)),
            jcc!(print_int_push_loop, Cnd::NE),
            cmpq!(imm!(0), reg!(RSI)),
            jcc!(print_int_print_loop, Cnd::EQ),
            pushq!(imm!(i64::from(b'-'))),
            jmp!(print_int_print_loop)
        ),
    );
    blocks.insert(
        print_int_print_loop,
        block!(
            // Print top of stack
            movq!(imm!(1), reg!(RAX)), // syscall 1: Write
            movq!(imm!(1), reg!(RDI)), // STDOUT
            movq!(reg!(RSP), reg!(RSI)),
            movq!(imm!(1), reg!(RDX)),
            syscall!(),
            // Check if we continue
            popq!(reg!(RAX)),
            cmpq!(imm!(i64::from(b'\n')), reg!(RAX)),
            jcc!(print_int_print_loop, Cnd::NE),
            jmp!(print_int_exit)
        ),
    );
    blocks.insert(print_int_exit, block!(retq!()));

    entry
}

fn add_read_block<'p>(
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>,
    exit: UniqueSym<'p>,
) -> UniqueSym<'p> {
    let entry = gen_sym("read_int");
    let read_int_is_neg = gen_sym("read_int_is_neg");
    let read_int_loop = gen_sym("read_int_loop");
    let read_int_first = gen_sym("read_int_first");
    let read_int_exit = gen_sym("read_int_exit");
    let read_int_neg = gen_sym("read_int_neg");
    let read_int_actual_exit = gen_sym("read_int_actual_exit");

    blocks.insert(
        entry,
        block!(
            pushq!(reg!(RBX)), // save a callee-saved register
            pushq!(reg!(R13)),
            movq!(imm!(0), reg!(R13)),
            movq!(imm!(0), reg!(RBX)), // zero out RBX
            subq!(imm!(8), reg!(RSP)), // allocate some space on the stack for reading the next byte
            // read initial character
            movq!(imm!(0), reg!(RAX)),   // READ = 0
            movq!(imm!(0), reg!(RDI)),   // STDIN = 0
            movq!(reg!(RSP), reg!(RSI)), // RSI is pointer to allocated byte
            movq!(imm!(1), reg!(RDX)),   // bytes to read = 1
            syscall!(),
            // check if first character is -
            movq!(deref!(RSP, 0), reg!(RAX)),
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'-')), reg!(RCX)),
            jcc!(read_int_is_neg, Cnd::EQ),
            jmp!(read_int_first)
        ),
    );
    blocks.insert(
        read_int_is_neg,
        block!(movq!(imm!(1), reg!(R13)), jmp!(read_int_loop)),
    );

    blocks.insert(
        read_int_loop,
        block!(
            movq!(imm!(0), reg!(RAX)),   // READ = 0
            movq!(imm!(0), reg!(RDI)),   // STDIN = 0
            movq!(reg!(RSP), reg!(RSI)), // RSI is pointer to allocated byte
            movq!(imm!(1), reg!(RDX)),   // bytes to read = 1
            syscall!(),
            jmp!(read_int_first)
        ),
    );

    blocks.insert(
        read_int_first,
        block!(
            movq!(deref!(RSP, 0), reg!(RAX)),
            // check if newline
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'\n')), reg!(RCX)),
            jcc!(read_int_exit, Cnd::EQ),
            movq!(imm!(66), reg!(RDI)),
            // check if >b'9'
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'9')), reg!(RCX)),
            jcc!(exit, Cnd::GT),
            // check if <b'0'
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'0')), reg!(RCX)),
            jcc!(exit, Cnd::LT),
            movq!(imm!(10), reg!(RAX)),
            mulq!(reg!(RBX)),
            movq!(reg!(RAX), reg!(RBX)),
            movq!(deref!(RSP, 0), reg!(RAX)),
            subq!(imm!(i64::from(b'0')), reg!(RAX)),
            addq!(reg!(RAX), reg!(RBX)),
            jmp!(read_int_loop)
        ),
    );
    blocks.insert(
        read_int_exit,
        block!(
            cmpq!(imm!(0), reg!(R13)),
            jcc!(read_int_neg, Cnd::NE),
            jmp!(read_int_actual_exit)
        ),
    );

    blocks.insert(
        read_int_neg,
        block!(negq!(reg!(RBX)), jmp!(read_int_actual_exit)),
    );

    blocks.insert(
        read_int_actual_exit,
        block!(
            movq!(reg!(RBX), reg!(RAX)),
            addq!(imm!(8), reg!(RSP)),
            popq!(reg!(R13)),
            popq!(reg!(RBX)),
            retq!()
        ),
    );

    entry
}
