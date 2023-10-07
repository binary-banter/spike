mod binary;
mod push_pop;
mod unary;
mod special;

use push_pop::PushPopInfo;
use unary::UnaryOpInfo;
use crate::language::x86var::{Arg, Block, Instr, Reg, X86Program};
use crate::passes::emit::binary::{BinaryOpInfo, encode_binary_instr};

impl<'p> X86Program<'p> {
    pub fn emit(self) -> Vec<u8> {
        let mut machine_code = Vec::new();
        for (_name, block) in &self.blocks {
            emit_block(block, &mut machine_code);
        }
        machine_code
    }
}

fn emit_block(block: &Block<Arg>, machine_code: &mut Vec<u8>) {
    for instr in &block.instrs {
        emit_instr(instr, machine_code);
    }
}

fn emit_instr(instr: &Instr<Arg>, machine_code: &mut Vec<u8>) {
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
            ("_print_int", 1) => todo!(),
            ("_read_int", 0) => todo!(),
            (_lbl, _) => todo!(),
        },
        Instr::Retq => {
            vec![0xC3]
        }
        Instr::Jmp { lbl: _ } => todo!(),
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
            emit_instr(&$instr, &mut output);

            assert_eq!(output, $expected);
        }
    };
}
