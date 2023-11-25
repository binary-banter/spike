use crate::passes::select::Cnd;
use crate::passes::select::{Block, VarArg};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::{
    addq, block, cmpq, deref, divq, imm, jcc, jmp, movq, mulq, negq, popq, pushq, reg, retq, subq,
    syscall,
};
use std::collections::HashMap;

pub type Std<'p> = HashMap<&'p str, UniqueSym<'p>>;

pub fn add_std_library<'p>(std: &Std<'p>, blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>) {
    add_exit_block(std["exit"], blocks);
    add_print_block(std["print"], blocks);
    add_read_block(std["read"], blocks, std["exit"]);
}

fn add_exit_block<'p>(
    entry: UniqueSym<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>,
) {
    blocks.insert(
        entry,
        block!(
            movq!(reg!(RAX), reg!(RDI)),
            movq!(imm!(0x3C), reg!(RAX)),
            syscall!(2)
        ),
    );
}

fn add_print_block<'p>(
    entry: UniqueSym<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>,
) {
    let print_neg = gen_sym("print_neg");
    let print_push_loop = gen_sym("print_push_loop");
    let print_print_loop = gen_sym("print_print_loop");
    let print_exit = gen_sym("print_exit");

    blocks.insert(
        entry,
        block!(
            pushq!(reg!(RAX)),
            movq!(imm!(10), reg!(RCX)),
            pushq!(imm!(i64::from(b'\n'))),
            movq!(imm!(0), reg!(RSI)),
            cmpq!(imm!(0), reg!(RAX)),
            jcc!(print_neg, Cnd::Sign),
            jmp!(print_push_loop)
        ),
    );
    blocks.insert(
        print_neg,
        block!(
            movq!(imm!(1), reg!(RSI)),
            negq!(reg!(RAX)),
            jmp!(print_push_loop)
        ),
    );
    blocks.insert(
        print_push_loop,
        block!(
            movq!(imm!(0), reg!(RDX)),
            divq!(reg!(RCX)),
            addq!(imm!(i64::from(b'0')), reg!(RDX)),
            pushq!(reg!(RDX)),
            cmpq!(imm!(0), reg!(RAX)),
            jcc!(print_push_loop, Cnd::NE),
            cmpq!(imm!(0), reg!(RSI)),
            jcc!(print_print_loop, Cnd::EQ),
            pushq!(imm!(i64::from(b'-'))),
            jmp!(print_print_loop)
        ),
    );
    blocks.insert(
        print_print_loop,
        block!(
            // Print top of stack
            movq!(imm!(1), reg!(RAX)), // syscall 1: Write
            movq!(imm!(1), reg!(RDI)), // STDOUT
            movq!(reg!(RSP), reg!(RSI)),
            movq!(imm!(1), reg!(RDX)),
            syscall!(4),
            // Check if we continue
            popq!(reg!(RAX)),
            cmpq!(imm!(i64::from(b'\n')), reg!(RAX)),
            jcc!(print_print_loop, Cnd::NE),
            jmp!(print_exit)
        ),
    );
    blocks.insert(print_exit, block!(popq!(reg!(RAX)), retq!()));
}

fn add_read_block<'p>(
    entry: UniqueSym<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg>>,
    exit: UniqueSym<'p>,
) {
    let read_is_neg = gen_sym("read_is_neg");
    let read_loop = gen_sym("read_loop");
    let read_first = gen_sym("read_first");
    let read_exit = gen_sym("read_exit");
    let read_neg = gen_sym("read_neg");
    let read_actual_exit = gen_sym("read_actual_exit");

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
            syscall!(4),
            // check if first character is -
            movq!(deref!(RSP, 0), reg!(RAX)),
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'-')), reg!(RCX)),
            jcc!(read_is_neg, Cnd::EQ),
            jmp!(read_first)
        ),
    );
    blocks.insert(
        read_is_neg,
        block!(movq!(imm!(1), reg!(R13)), jmp!(read_loop)),
    );

    blocks.insert(
        read_loop,
        block!(
            movq!(imm!(0), reg!(RAX)),   // READ = 0
            movq!(imm!(0), reg!(RDI)),   // STDIN = 0
            movq!(reg!(RSP), reg!(RSI)), // RSI is pointer to allocated byte
            movq!(imm!(1), reg!(RDX)),   // bytes to read = 1
            syscall!(4),
            jmp!(read_first)
        ),
    );

    blocks.insert(
        read_first,
        block!(
            movq!(deref!(RSP, 0), reg!(RAX)),
            // check if newline
            movq!(reg!(RAX), reg!(RCX)),
            cmpq!(imm!(i64::from(b'\n')), reg!(RCX)),
            jcc!(read_exit, Cnd::EQ),
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
            jmp!(read_loop)
        ),
    );
    blocks.insert(
        read_exit,
        block!(
            cmpq!(imm!(0), reg!(R13)),
            jcc!(read_neg, Cnd::NE),
            jmp!(read_actual_exit)
        ),
    );

    blocks.insert(read_neg, block!(negq!(reg!(RBX)), jmp!(read_actual_exit)));

    blocks.insert(
        read_actual_exit,
        block!(
            movq!(reg!(RBX), reg!(RAX)),
            addq!(imm!(8), reg!(RSP)),
            popq!(reg!(R13)),
            popq!(reg!(RBX)),
            retq!()
        ),
    );
}
