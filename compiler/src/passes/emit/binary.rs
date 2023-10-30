use crate::passes::emit::encode_reg;
use crate::passes::interference::Arg;
use crate::passes::select::Reg;

pub struct BinaryOpInfo {
    /// Opcode when src = Reg and dst = Reg | Deref.
    pub r_rm: u8,
    /// Opcode when src = Reg | Deref and dst = Reg.
    pub rm_r: u8,
    /// Opcode when src = Imm and dst = Reg | Deref.
    pub i_rm: u8,
    /// Padding to use instead of the src operand when src = Imm.
    pub pad: u8,
}

pub const ADDQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x01,
    rm_r: 0x03,
    i_rm: 0x81,
    pad: 0x00,
};

pub const SUBQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x29,
    rm_r: 0x2B,
    i_rm: 0x81,
    pad: 0x05,
};

pub const CMPQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x39,
    rm_r: 0x3B,
    i_rm: 0x81,
    pad: 0x07,
};

pub const ANDQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x21,
    rm_r: 0x23,
    i_rm: 0x81,
    pad: 0x04,
};

pub const ORQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x09,
    rm_r: 0x0B,
    i_rm: 0x81,
    pad: 0x01,
};

pub const XORQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x31,
    rm_r: 0x33,
    i_rm: 0x81,
    pad: 0x06,
};

pub const MOVQ_INFO: BinaryOpInfo = BinaryOpInfo {
    r_rm: 0x89,
    rm_r: 0x8B,
    i_rm: 0xC7,
    pad: 0x00,
};

pub fn encode_binary_instr(op_info: BinaryOpInfo, src: &Arg, dst: &Arg) -> Vec<u8> {
    match (src, dst) {
        (Arg::Reg { reg: src }, Arg::Reg { reg: dst }) => {
            let (s, sss) = encode_reg(src);
            let (d, ddd) = encode_reg(dst);
            vec![
                0b0100_1000 | (s << 2) | d,
                op_info.r_rm,
                0b11_000_000 | sss << 3 | ddd,
            ]
        }
        (Arg::Deref { reg: src, off }, Arg::Reg { reg: dst }) => {
            let (s, sss) = encode_reg(src);
            let (d, ddd) = encode_reg(dst);
            let off = *off as i32;

            let mut v = vec![
                0b0100_1000 | (d << 2) | s,
                op_info.rm_r,
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
                op_info.r_rm,
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
                op_info.i_rm,
                0b11_000_000 | op_info.pad << 3 | ddd,
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
                op_info.i_rm,
                0b10_000_000 | op_info.pad << 3 | ddd,
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
    use crate::*;

    mod add {
        use super::*;

        check!(reg_reg, addq!(reg!(RSP), reg!(RDX)), vec![0x48, 0x01, 0xE2]);
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

        check!(reg_reg, subq!(reg!(RSP), reg!(RDX)), vec![0x48, 0x29, 0xE2]);
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
