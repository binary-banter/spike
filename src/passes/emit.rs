use crate::language::x86var::{Arg, Block, Instr, Reg, X86Program};

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
        Instr::Negq { dst } => encode_unary_instr(
            UnaryOpInfo {
                op: 0xF7,
                imm_as_src: 0x3,
            },
            dst,
        ),
        Instr::Pushq { src: _ } => todo!(),
        Instr::Popq { dst: _ } => todo!(),
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

struct UnaryOpInfo {
    op: u8,
    imm_as_src: u8,
}

fn encode_unary_instr(op_info: UnaryOpInfo, dst: &Arg) -> Vec<u8> {
    match dst {
        Arg::Reg { reg: dst } => {
            // use: REX.W + opcode /r
            let (d, ddd) = encode_reg(dst);
            vec![
                0b0100_1000 | d,
                op_info.op,
                0b11_000_000 | op_info.imm_as_src << 3 | ddd,
            ]
        }
        Arg::Deref { reg: dst, off } => {
            // use: REX.W + opcode /r
            let (d, ddd) = encode_reg(dst);
            let off = *off as i32;

            // 10 011 100
            let mut v = vec![
                0b0100_1000 | d,
                op_info.op,
                0b10_000_000 | op_info.imm_as_src << 3 | ddd,
            ];

            if matches!(dst, Reg::RSP | Reg::R12) {
                v.push(0x24);
            }
            v.extend(off.to_le_bytes());
            v
        }
        Arg::Imm { .. } => unreachable!(),
    }
}

struct BinaryOpInfo {
    /// Opcode in case the binary operation performs op(src: reg, dst: reg).
    op_reg_reg: u8,
    /// Opcode in case the binary operation performs op(src: imm, dst: reg).
    op_imm_reg: u8,
    /// Opcode in case the binary operation performs op(src: deref, dst: reg).
    op_deref_reg: u8,
    /// Opcode in case the binary operation performs op(src: deref, dst: reg).
    op_reg_deref: u8,

    op_imm_deref: u8,
    /// Bits to use instead of the absent src
    imm_as_src: u8,
}

fn encode_binary_instr(op_info: BinaryOpInfo, src: &Arg, dst: &Arg) -> Vec<u8> {
    match (src, dst) {
        (Arg::Reg { reg: src }, Arg::Reg { reg: dst }) => {
            // use: REX.W + opcode /r
            let (s, sss) = encode_reg(src);
            let (d, ddd) = encode_reg(dst);
            vec![
                0b0100_1000 | (s << 2) | d,
                op_info.op_reg_reg,
                0b11_000_000 | sss << 3 | ddd,
            ]
        }
        (Arg::Deref { reg: src, off }, Arg::Reg { reg: dst }) => {
            let (s, sss) = encode_reg(src);
            let (d, ddd) = encode_reg(dst);
            let off = *off as i32;

            let mut v = vec![
                0b0100_1000 | (d << 2) | s,
                op_info.op_deref_reg,
                0b10_000_000 | ddd << 3 | sss,
            ];
            if matches!(src, Reg::RSP | Reg::R12) {
                v.push(0x24);
            }
            v.extend(off.to_le_bytes());
            v
        }
        (Arg::Reg { reg: src }, Arg::Deref { reg: dst, off }) => {
            let (s, sss) = encode_reg(src);
            let (d, ddd) = encode_reg(dst);
            let off = *off as i32;

            let mut v = vec![
                0b0100_1000 | (s << 2) | d,
                op_info.op_reg_deref,
                0b10_000_000 | sss << 3 | ddd,
            ];
            if matches!(src, Reg::RSP | Reg::R12) {
                v.push(0x24);
            }
            v.extend(off.to_le_bytes());
            v
        }
        (Arg::Imm { val: imm }, Arg::Reg { reg: dst }) => {
            let (d, ddd) = encode_reg(dst);
            let imm = *imm as i32;

            let mut v = vec![
                0b0100_1000 | d,
                op_info.op_imm_reg,
                0b11_000_000 | op_info.imm_as_src << 3 | ddd,
            ];
            v.extend(imm.to_le_bytes());
            v
        }
        (Arg::Imm { val: imm }, Arg::Deref { reg: dst, off }) => {
            let (d, ddd) = encode_reg(dst);
            let off = *off as i32;
            let imm = *imm as i32;

            let mut v = vec![
                0b0100_1000 | d,
                op_info.op_imm_deref,
                0b10_000_000 | op_info.imm_as_src << 3 | ddd,
            ];
            if matches!(dst, Reg::RSP | Reg::R12) {
                v.push(0x24);
            }
            v.extend(off.to_le_bytes());
            v.extend(imm.to_le_bytes());
            v
        }
        (Arg::Deref { .. }, Arg::Deref { .. }) => {
            unreachable!("Found binary instruction with 2 derefs.");
        }
        (_, Arg::Imm { .. }) => {
            unreachable!("Found immediate in destination position.");
        }
    }
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
mod tests {
    use crate::language::x86var::{Arg, Instr, Reg};

    macro_rules! check {
        ($name: ident, $instr: expr, $expected: expr) => {
            #[test]
            fn $name() {
                let mut output = vec![];
                super::emit_instr(&$instr, &mut output);

                println!("Got: {output:x?}");
                println!("Exp: {:x?}", $expected);

                assert_eq!(output, $expected);
            }
        };
    }

    check!(retq, Instr::Retq, vec![0xC3]);

    check!(
        add_reg_reg,
        Instr::Addq {
            src: Arg::Reg { reg: Reg::RSP },
            dst: Arg::Reg { reg: Reg::RDX }
        },
        vec![0x48, 0x01, 0xE2]
    );
    check!(
        add_imm_reg,
        Instr::Addq {
            src: Arg::Imm {
                val: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RBP }
        },
        vec![0x48, 0x81, 0xC5, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        add_deref_reg1,
        Instr::Addq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RDI }
        },
        vec![0x48, 0x03, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        add_deref_reg2,
        Instr::Addq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::R15 }
        },
        vec![0x4C, 0x03, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        add_reg_deref1,
        Instr::Addq {
            src: Arg::Reg { reg: Reg::RCX },
            dst: Arg::Deref {
                reg: Reg::R15,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0x01, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        add_imm_deref1,
        Instr::Addq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::R9,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0x81, 0x81, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );
    check!(
        add_imm_deref2,
        Instr::Addq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::RDX,
                off: i32::MAX as i64
            }
        },
        vec![0x48, 0x81, 0x82, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );

    check!(
        sub_reg_reg,
        Instr::Subq {
            src: Arg::Reg { reg: Reg::RSP },
            dst: Arg::Reg { reg: Reg::RDX }
        },
        vec![0x48, 0x29, 0xE2]
    );
    check!(
        sub_imm_reg,
        Instr::Subq {
            src: Arg::Imm {
                val: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RBP }
        },
        vec![0x48, 0x81, 0xED, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        sub_deref_reg1,
        Instr::Subq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RDI }
        },
        vec![0x48, 0x2B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        sub_deref_reg2,
        Instr::Subq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::R15 }
        },
        vec![0x4C, 0x2B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        sub_reg_deref1,
        Instr::Subq {
            src: Arg::Reg { reg: Reg::RCX },
            dst: Arg::Deref {
                reg: Reg::R15,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0x29, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        sub_imm_deref1,
        Instr::Subq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::R9,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0x81, 0xA9, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );
    check!(
        sub_imm_deref2,
        Instr::Subq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::RDX,
                off: i32::MAX as i64
            }
        },
        vec![0x48, 0x81, 0xAA, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );

    check!(
        mov_reg_reg,
        Instr::Movq {
            src: Arg::Reg { reg: Reg::RSP },
            dst: Arg::Reg { reg: Reg::RDX }
        },
        vec![0x48, 0x89, 0xE2]
    );
    check!(
        mov_imm_reg,
        Instr::Movq {
            src: Arg::Imm {
                val: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RBP }
        },
        vec![0x48, 0xC7, 0xC5, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        mov_deref_reg1,
        Instr::Movq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::RDI }
        },
        vec![0x48, 0x8B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        mov_deref_reg2,
        Instr::Movq {
            src: Arg::Deref {
                reg: Reg::RBX,
                off: i32::MAX as i64
            },
            dst: Arg::Reg { reg: Reg::R15 }
        },
        vec![0x4C, 0x8B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        mov_reg_deref1,
        Instr::Movq {
            src: Arg::Reg { reg: Reg::RCX },
            dst: Arg::Deref {
                reg: Reg::R15,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0x89, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
    );
    check!(
        mov_imm_deref1,
        Instr::Movq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::R9,
                off: i32::MAX as i64
            }
        },
        vec![0x49, 0xC7, 0x81, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );
    check!(
        mov_imm_deref2,
        Instr::Movq {
            src: Arg::Imm {
                val: (i32::MAX - 0xFF) as i64
            },
            dst: Arg::Deref {
                reg: Reg::RDX,
                off: i32::MAX as i64
            }
        },
        vec![0x48, 0xC7, 0x82, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
    );

    check!(
        neg_reg1,
        Instr::Negq {
            dst: Arg::Reg { reg: Reg::RSP }
        },
        vec![0x48, 0xF7, 0xDC]
    );
    check!(
        neg_reg2,
        Instr::Negq {
            dst: Arg::Reg { reg: Reg::R13 }
        },
        vec![0x49, 0xF7, 0xDD]
    );
    check!(
        neg_deref,
        Instr::Negq {
            dst: Arg::Deref {
                reg: Reg::RSP,
                off: i32::MAX as i64
            }
        },
        vec![0x48, 0xF7, 0x9C, 0x24, 0xFF, 0xFF, 0xFF, 0x7F]
    );
}
