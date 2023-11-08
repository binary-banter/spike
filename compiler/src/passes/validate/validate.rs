use crate::passes::parse::PrgParsed;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::NoMain;
use crate::passes::validate::PrgValidated;
use crate::utils::expect::expect;

impl<'p> PrgParsed<'p> {
    pub fn validate(self) -> Result<PrgValidated<'p>, TypeError> {
        let program = self.uniquify()?;
        let assignments = program.generate_constraints().solve()?;
        let program = program.resolve_types(assignments);
        program.check_sized()?;
        Ok(program)
    }
}

#[cfg(test)]
mod tests {
    use crate::passes::parse::parse::parse_program;
    use test_each_file::test_each_file;

    fn validate([test]: [&str; 1], should_fail: bool) {
        let res = parse_program(test).unwrap().validate();

        match (res, should_fail) {
            (Ok(_), true) => panic!("Program should not pass type-checking."),
            (Err(e), false) => {
                panic!("Program should have passed type-checking, but returned error: '{e}'.")
            }
            _ => {}
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as validate_succeed => |p| validate(p, false) }
    test_each_file! { for ["test"] in "./programs/fail/type_check" as validate_fail => |p| validate(p, true) }
}
