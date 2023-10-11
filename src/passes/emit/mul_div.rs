use crate::language::x86var::Arg;
use crate::passes::emit::encode_reg;

pub struct MulDivOpInfo {
    pub op: u8,
    pub imm_as_src: u8,
}

pub fn encode_muldiv_instr(op_info: MulDivOpInfo, reg: &Arg) -> Vec<u8> {
    match reg {
        Arg::Imm { .. } => todo!(),
        Arg::Reg { reg } => {
            let (d, ddd) = encode_reg(reg);
            vec![
                0b0100_1000 | d,
                op_info.op,
                0b11_000_000 | op_info.imm_as_src << 3 | ddd,
            ]
        }
        Arg::Deref { .. } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    mod division {
        use crate::*;

        check!(div1, divq!(reg!(R15)), vec![0x49, 0xF7, 0xF7]);
        check!(div2, divq!(reg!(RDX)), vec![0x48, 0xF7, 0xF2]);
    }

    mod multiply {
        use crate::*;

        check!(mul1, mulq!(reg!(R15)), vec![0x49, 0xF7, 0xE7]);
        check!(mul2, mulq!(reg!(RDX)), vec![0x48, 0xF7, 0xE2]);
    }
}
