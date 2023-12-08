use crate::passes::select::Cnd;

pub fn encode_setcc(cnd: &Cnd) -> Vec<u8> {
    let cnd = match cnd {
        Cnd::EQ => 0x94,
        Cnd::NE => 0x95,
        Cnd::LT => 0x9C,
        Cnd::LE => 0x9E,
        Cnd::GE => 0x9D,
        Cnd::GT => 0x9F,
        _ => todo!("Implement other flag codes."),
    };

    vec![0x0F, cnd, 0xC0]
}

// #[cfg(test)]
// mod tests {
//     mod retq {
//
//         use crate::*;
//
//         check!(retq, ret!(), vec![0xC3]);
//     }
// }
