mod binary;
mod push_pop;
mod unary;
mod special;
mod io;

use std::collections::HashMap;
use push_pop::PushPopInfo;
use unary::UnaryOpInfo;
use crate::language::x86var::{Arg, Block, Instr, Reg, X86Program};
use crate::*;
use crate::passes::emit::binary::{BinaryOpInfo, encode_binary_instr};
use crate::passes::emit::io::add_io_blocks;

impl<'p> X86Program<'p> {
    pub fn emit(mut self) -> (usize, Vec<u8>) {
        add_io_blocks(&mut self.blocks);

        let mut machine_code = Vec::new();

        let mut jumps: HashMap<usize, &'p str> = HashMap::new();
        let mut offsets= HashMap::new();

        for (name, block) in &self.blocks {
            offsets.insert(name, machine_code.len());
            emit_block(block, &mut machine_code, &mut jumps);
        }

        dbg!(&offsets);

        for (addr, block) in jumps {
            let src = (addr + 4) as i32;
            let target = offsets[&block] as i32;
            let jump = target - src;

            machine_code[addr .. addr + 4].copy_from_slice(&jump.to_le_bytes());
        }

        (offsets[&"main"], machine_code)
    }
}

fn emit_block<'p>(block: &Block<'p, Arg>, machine_code: &mut Vec<u8>, jumps: &mut HashMap<usize, &'p str>) {
    for instr in &block.instrs {
        emit_instr(instr, machine_code, jumps);
    }
}

fn emit_instr<'p>(instr: &Instr<'p, Arg>, machine_code: &mut Vec<u8>, jumps: &mut HashMap<usize, &'p str>) {
    let v = match instr {
        Instr::Addq { src, dst } => encode_binary_instr(
            BinaryOpInfo {
                op_reg_reg: 0x01,
                op_reg_deref: 0x01,
                op_imm_deref: 0x81,
                op_imm_reg: 0x81,
                op_deref_reg: 0x03,
                imm_as_src: 0x0,
            },
            src,
            dst,
        ),
        Instr::Subq { src, dst } => encode_binary_instr(
            BinaryOpInfo {
                op_reg_reg: 0x29,
                op_imm_reg: 0x81,
                op_deref_reg: 0x2B,
                op_reg_deref: 0x29,
                op_imm_deref: 0x81,
                imm_as_src: 0x5,
            },
            src,
            dst,
        ),
        Instr::Movq { src, dst } => encode_binary_instr(
            BinaryOpInfo {
                op_reg_reg: 0x89,
                op_imm_reg: 0xC7,
                op_deref_reg: 0x8B,
                op_reg_deref: 0x89,
                op_imm_deref: 0xC7,
                imm_as_src: 0x0,
            },
            src,
            dst,
        ),
        Instr::Negq { dst } => unary::encode_unary_instr(
            UnaryOpInfo {
                op: 0xF7,
                imm_as_src: 0x3,
            },
            dst,
        ),
        Instr::Pushq { src } => {
            push_pop::encode_push_pop(PushPopInfo {
                op_reg: 0x50,
                op_deref: 0xFF,
                op_imm: 0x68,
                imm_as_src: 0x6,
            }, src)
        },
        Instr::Popq { dst } => {
            push_pop::encode_push_pop(PushPopInfo {
                op_reg: 0x58,
                op_deref: 0x8F,
                op_imm: 0, //Unreachable
                imm_as_src: 0x0,
            }, dst)
        },
        Instr::Callq { lbl, arity } => match (*lbl, arity) {
            // ("_print_int", 1) => {
            //     todo!()
            // },
            // ("_read_int", 0) => todo!(),
            // ("exit", 1) => {
            //     let mut v = vec![];
            //     emit_instr(&movq!(imm!(0x3C), reg!(RAX)), &mut v, jumps);
            //     v.extend([0x0F, 0x05]);
            //     v
            // }
            (lbl, _) => {
                jumps.insert(machine_code.len() + 1, lbl);
                vec![0xE8, 0x00, 0x00, 0x00, 0x00]
            },
        },
        Instr::Jmp { lbl } => {
            jumps.insert(machine_code.len() + 1, lbl);
            vec![0xE9, 0x00, 0x00, 0x00, 0x00]
        },
        Instr::Retq => {
            vec![0xC3]
        }
        Instr::Syscall => {
            vec![0x0F, 0x05]
        }
        Instr::Divq { divisor } => {
            match divisor {
                Arg::Imm { .. } => todo!(),
                Arg::Reg { reg: divisor } => {
                    let (d, ddd) = encode_reg(divisor);
                    vec![
                        0b0100_1000 | d,
                        0xF7,
                        0b11_000_000 | 0b110 << 3 | ddd,
                    ]
                }
                Arg::Deref { .. } => todo!(),
            }
        }
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

#[cfg(test)]
#[macro_export]
macro_rules! check {
    ($name: ident, $instr: expr, $expected: expr) => {
        #[test]
        fn $name() {
            let mut output = vec![];
            use crate::passes::emit::emit_instr;
            use std::collections::HashMap;
            emit_instr(&$instr, &mut output, &mut HashMap::new());

            assert_eq!(output, $expected);
        }
    };
}
