#[cfg(test)]
mod tests {
    mod retq {
        use crate::language::x86var::Instr;
        use crate::*;

        check!(retq, retq!(), vec![0xC3]);
    }
}
