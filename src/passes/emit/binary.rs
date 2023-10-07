use crate::language::x86var::Arg;
use crate::passes::emit::encode_reg;
use crate::passes::emit::Reg;

pub struct BinaryOpInfo {
    /// Opcode in case the binary operation performs op(src: reg, dst: reg).
    pub op_reg_reg: u8,
    /// Opcode in case the binary operation performs op(src: imm, dst: reg).
    pub op_imm_reg: u8,
    /// Opcode in case the binary operation performs op(src: deref, dst: reg).
    pub op_deref_reg: u8,
    /// Opcode in case the binary operation performs op(src: reg, dst: deref).
    pub op_reg_deref: u8,
    /// Opcode in case the binary operation performs op(src: imm, dst: deref).
    pub op_imm_deref: u8,
    /// Bits to use instead of the absent src
    pub imm_as_src: u8,
}

pub fn encode_binary_instr(op_info: BinaryOpInfo, src: &Arg, dst: &Arg) -> Vec<u8> {
    match (src, dst) {
        (Arg::Reg { reg: src }, Arg::Reg { reg: dst }) => {
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

#[cfg(test)]
mod tests {
    use crate::language::x86var::{Arg, Instr, Reg};
    use crate::*;

    mod add {
        use super::*;

        check!(
            reg_reg,
            addq!(reg!(RSP), reg!(RDX)),
            vec![0x48, 0x01, 0xE2]
        );
        check!(
            imm_reg,
            addq!(imm!(i32::MAX as i64), reg!(RBP)),
            vec![0x48, 0x81, 0xC5, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg1,
            addq!(deref!(RBX, i32::MAX as i64), reg!(RDI)),
            vec![0x48, 0x03, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg2,
            addq!(deref!(RBX, i32::MAX as i64), reg!(R15)),
            vec![0x4C, 0x03, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            reg_deref1,
            addq!(reg!(RCX), deref!(R15, i32::MAX as i64)),
            vec![0x49, 0x01, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref1,
            addq!(imm!((i32::MAX - 0xFF) as i64), deref!(R9, i32::MAX as i64)),
            vec![0x49, 0x81, 0x81, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref2,
            addq!(imm!((i32::MAX - 0xFF) as i64), deref!(RDX, i32::MAX as i64)),
            vec![0x48, 0x81, 0x82, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
    }

    mod sub {
        use super::*;

        check!(
            reg_reg,
            subq!(reg!(RSP), reg!(RDX)),
            vec![0x48, 0x29, 0xE2]
        );
        check!(
            imm_reg,
            subq!(imm!(i32::MAX as i64), reg!(RBP)),
            vec![0x48, 0x81, 0xED, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg1,
            subq!(deref!(RBX, i32::MAX as i64), reg!(RDI)),
            vec![0x48, 0x2B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg2,
            subq!(deref!(RBX, i32::MAX as i64), reg!(R15)),
            vec![0x4C, 0x2B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            reg_deref1,
            subq!(reg!(RCX), deref!(R15, i32::MAX as i64)),
            vec![0x49, 0x29, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref1,
            subq!(imm!((i32::MAX - 0xFF) as i64), deref!(R9, i32::MAX as i64)),
            vec![0x49, 0x81, 0xA9, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref2,
            subq!(imm!((i32::MAX - 0xFF) as i64), deref!(RDX, i32::MAX as i64)),
            vec![0x48, 0x81, 0xAA, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
    }

    mod r#move {
        use super::*;

        check!(reg_reg, movq!(reg!(RSP), reg!(RDX)), vec![0x48, 0x89, 0xE2]);
        check!(
            imm_reg,
            movq!(imm!(i32::MAX as i64), reg!(RBP)),
            vec![0x48, 0xC7, 0xC5, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg1,
            movq!(deref!(RBX, i32::MAX as i64), reg!(RDI)),
            vec![0x48, 0x8B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            deref_reg2,
            movq!(deref!(RBX, i32::MAX as i64), reg!(R15)),
            vec![0x4C, 0x8B, 0xBB, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            reg_deref1,
            movq!(reg!(RCX), deref!(R15, i32::MAX as i64)),
            vec![0x49, 0x89, 0x8F, 0xFF, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref1,
            movq!(imm!((i32::MAX - 0xFF) as i64), deref!(R9, i32::MAX as i64)),
            vec![0x49, 0xC7, 0x81, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
        check!(
            imm_deref2,
            movq!(imm!((i32::MAX - 0xFF) as i64), deref!(RDX, i32::MAX as i64)),
            vec![0x48, 0xC7, 0x82, 0xFF, 0xFF, 0xFF, 0x7F, 0x00, 0xFF, 0xFF, 0x7F]
        );
    }
}
