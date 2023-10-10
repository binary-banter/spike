//! This code will assemble the instructions into a sequence of bytes (machine code).

mod binary;
mod io;
mod mul_div;
mod push_pop;
mod special;
mod unary;

use crate::language::x86var::{Arg, Block, Cnd, Instr, Reg, X86Program};
use crate::passes::emit::binary::{encode_binary_instr, ADDQ_INFO, MOVQ_INFO, SUBQ_INFO};
use crate::passes::emit::mul_div::{encode_muldiv_instr, MulDivOpInfo};
use crate::passes::emit::push_pop::{encode_push_pop, POPQ_INFO, PUSHQ_INFO};
use crate::passes::emit::unary::{encode_unary_instr, NEGQ_INFO};
use std::collections::HashMap;

impl<'p> X86Program<'p> {
    //! See module-level documentation.
    pub fn emit(mut self) -> (usize, Vec<u8>) {
        let mut machine_code = Vec::new();

        let mut jumps: HashMap<usize, &'p str> = HashMap::new();
        let mut offsets = HashMap::new();

        for (name, block) in &self.blocks {
            offsets.insert(name, machine_code.len());
            emit_block(block, &mut machine_code, &mut jumps);
        }

        for (addr, block) in jumps {
            let src = (addr + 4) as i32;
            let target = offsets[&block] as i32;
            let jump = target - src;

            machine_code[addr..addr + 4].copy_from_slice(&jump.to_le_bytes());
        }

        (offsets[&"main"], machine_code)
    }
}

fn emit_block<'p>(
    block: &Block<'p, Arg>,
    machine_code: &mut Vec<u8>,
    jumps: &mut HashMap<usize, &'p str>,
) {
    for instr in &block.instrs {
        emit_instr(instr, machine_code, jumps);
    }
}

fn emit_instr<'p>(
    instr: &Instr<'p, Arg>,
    machine_code: &mut Vec<u8>,
    jumps: &mut HashMap<usize, &'p str>,
) {
    let v = match instr {
        Instr::Addq { src, dst } => encode_binary_instr(ADDQ_INFO, src, dst),
        Instr::Subq { src, dst } => encode_binary_instr(SUBQ_INFO, src, dst),
        Instr::Movq { src, dst } => encode_binary_instr(MOVQ_INFO, src, dst),
        Instr::Negq { dst } => encode_unary_instr(NEGQ_INFO, dst),
        Instr::Pushq { src } => encode_push_pop(PUSHQ_INFO, src),
        Instr::Popq { dst } => encode_push_pop(POPQ_INFO, dst),
        Instr::Callq { lbl, .. } => {
            jumps.insert(machine_code.len() + 1, lbl);
            vec![0xE8, 0x00, 0x00, 0x00, 0x00]
        }
        Instr::Jmp { lbl } => {
            jumps.insert(machine_code.len() + 1, lbl);
            vec![0xE9, 0x00, 0x00, 0x00, 0x00]
        }
        Instr::Jcc { lbl, cnd } => {
            jumps.insert(machine_code.len() + 2, lbl);
            vec![0x0F, encode_cnd(cnd), 0x00, 0x00, 0x00, 0x00]
        }
        Instr::Retq => vec![0xC3],
        Instr::Syscall => vec![0x0F, 0x05],
        Instr::Divq { divisor } => encode_muldiv_instr(
            MulDivOpInfo {
                op: 0xF7,
                imm_as_src: 0b110,
            },
            divisor,
        ),
        Instr::Mulq { src } => encode_muldiv_instr(
            MulDivOpInfo {
                op: 0xF7,
                imm_as_src: 0b100,
            },
            src,
        ),
    };
    machine_code.extend(v);
}

fn encode_reg(reg: &Reg) -> (u8, u8) {
    match reg {
        Reg::RAX => (0, 0),
        Reg::RCX => (0, 1),
        Reg::RDX => (0, 2),
        Reg::RBX => (0, 3),
        Reg::RSP => (0, 4),
        Reg::RBP => (0, 5),
        Reg::RSI => (0, 6),
        Reg::RDI => (0, 7),
        Reg::R8 => (1, 0),
        Reg::R9 => (1, 1),
        Reg::R10 => (1, 2),
        Reg::R11 => (1, 3),
        Reg::R12 => (1, 4),
        Reg::R13 => (1, 5),
        Reg::R14 => (1, 6),
        Reg::R15 => (1, 7),
    }
}

fn encode_cnd(cnd: &Cnd) -> u8 {
    match cnd {
        Cnd::Above => 0x87,
        Cnd::AboveOrEqual | Cnd::NotCarry => 0x83,
        Cnd::Below | Cnd::Carry => 0x82,
        Cnd::BelowOrEqual => 0x86,
        Cnd::Equal => 0x84,
        Cnd::Greater => 0x8F,
        Cnd::GreaterOrEqual => 0x8D,
        Cnd::Less => 0x8C,
        Cnd::LessOrEqual => 0x8E,
        Cnd::NotEqual => 0x85,
        Cnd::NotOverflow => 0x81,
        Cnd::ParityOdd => 0x8B,
        Cnd::NotSign => 0x89,
        Cnd::Overflow => 0x80,
        Cnd::ParityEven => 0x8A,
        Cnd::Sign => 0x88,
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! check {
    ($name: ident, $instr: expr, $expected: expr) => {
        #[test]
        fn $name() {
            let mut output = vec![];
            use std::collections::HashMap;
            use $crate::passes::emit::emit_instr;
            emit_instr(&$instr, &mut output, &mut HashMap::new());

            assert_eq!(output, $expected);
        }
    };
}
