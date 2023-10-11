// use crate::language::x86var::Arg;
// use crate::passes::emit;
// use crate::passes::emit::Reg;
//
// pub struct UnaryOpInfo {
//     pub op: u8,
//     pub imm_as_src: u8,
// }
//
// pub const NEGQ_INFO: UnaryOpInfo = UnaryOpInfo {
//     op: 0xF7,
//     imm_as_src: 0x3,
// };
//
// pub fn encode_unary_instr(op_info: UnaryOpInfo, dst: &Arg) -> Vec<u8> {
//     match dst {
//         Arg::Reg { reg: dst } => {
//             // use: REX.W + opcode /r
//             let (d, ddd) = emit::encode_reg(dst);
//             vec![
//                 0b0100_1000 | d,
//                 op_info.op,
//                 0b11_000_000 | op_info.imm_as_src << 3 | ddd,
//             ]
//         }
//         Arg::Deref { reg: dst, off } => {
//             // use: REX.W + opcode /r
//             let (d, ddd) = emit::encode_reg(dst);
//             let off = *off as i32;
//
//             // 10 011 100
//             let mut v = vec![
//                 0b0100_1000 | d,
//                 op_info.op,
//                 0b10_000_000 | op_info.imm_as_src << 3 | ddd,
//             ];
//
//             if matches!(dst, Reg::RSP | Reg::R12) {
//                 v.push(0x24);
//             }
//             v.extend(off.to_le_bytes());
//             v
//         }
//         Arg::Imm { .. } => unreachable!(),
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     mod neg {
//         use crate::language::x86var::Reg;
//         use crate::*;
//
//         check!(nreg1, negq!(reg!(RSP)), vec![0x48, 0xF7, 0xDC]);
//         check!(reg2, negq!(reg!(R13)), vec![0x49, 0xF7, 0xDD]);
//         check!(
//             deref,
//             negq!(deref!(RSP, i32::MAX as i64)),
//             vec![0x48, 0xF7, 0x9C, 0x24, 0xFF, 0xFF, 0xFF, 0x7F]
//         );
//     }
// }
