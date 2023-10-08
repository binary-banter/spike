#[cfg(test)]
mod tests {
    mod retq {
        use crate::language::x86var::{Instr};
        use crate::*;

        check!(retq, retq!(), vec![0xC3]);
    }

    mod division {
        use crate::language::x86var::{Instr};
        use crate::*;
        use crate::passes::emit::Arg;
        use crate::passes::emit::Reg;

        check!(div1, divq!(reg!(R15)), vec![0x49, 0xF7, 0xF7]);
        check!(div2, divq!(reg!(RDX)), vec![0x48, 0xF7, 0xF2]);
    }
}