use crate::passes::parse::PrgParsed;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgValidated;

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
    use crate::passes::validate::error::TypeError;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn validate([test]: [&str; 1]) {
        let (_, _, _, expected_error) = split_test(test);

        let result = parse_program(test).unwrap().validate();

        match (result, expected_error) {
            (Ok(_), None) => {}
            (Err(error), None) => {
                panic!("Should have succeeded, but panicked with `{error}` instead")
            }
            (Ok(_), Some(expected_error)) => {
                panic!("Expected error `{expected_error}`, but succeeded instead.")
            }
            (Err(error), Some(expected_error)) => match error {
                TypeError::UndeclaredVar { .. } => {
                    assert_eq!(expected_error, "UndeclaredVar")
                }
                TypeError::MismatchedType { .. } => {
                    assert_eq!(expected_error, "MismatchedType")
                }
                TypeError::TypeMismatchExpectFn { .. } => {
                    assert_eq!(expected_error, "TypeMismatchExpectFn")
                }
                TypeError::MismatchedTypes { .. } => {
                    assert_eq!(expected_error, "MismatchedTypes")
                }
                TypeError::DuplicateFunction { .. } => {
                    assert_eq!(expected_error, "DuplicateFunction")
                }
                TypeError::DuplicateArg { .. } => {
                    assert_eq!(expected_error, "DuplicateArg")
                }
                TypeError::ArgCountMismatch { .. } => {
                    assert_eq!(expected_error, "ArgCountMismatch")
                }
                TypeError::BreakOutsideLoop => {
                    assert_eq!(expected_error, "BreakOutsideLoop")
                }
                TypeError::ModifyImmutable { .. } => {
                    assert_eq!(expected_error, "ModifyImmutable")
                }
                TypeError::VariableShouldBeExpr { .. } => {
                    assert_eq!(expected_error, "VariableShouldBeExpr")
                }
                TypeError::VariableShouldBeStruct { .. } => {
                    assert_eq!(expected_error, "VariableShouldBeStruct")
                }
                TypeError::UnknownStructField { .. } => {
                    assert_eq!(expected_error, "UnknownStructField")
                }
                TypeError::VariableConstructMissingField { .. } => {
                    assert_eq!(expected_error, "VariableConstructMissingField")
                }
                TypeError::VariableConstructDuplicateField { .. } => {
                    assert_eq!(expected_error, "VariableConstructDuplicateField")
                }
                TypeError::TypeShouldBeStruct { .. } => {
                    assert_eq!(expected_error, "TypeShouldBeStruct")
                }
                TypeError::UnsizedType { .. } => {
                    assert_eq!(expected_error, "UnsizedType")
                }
                TypeError::IntegerOutOfBounds { .. } => {
                    assert_eq!(expected_error, "IntegerOutOfBounds")
                }
                TypeError::NoMain => {
                    assert_eq!(expected_error, "NoMain")
                }
            },
        }
    }

    test_each_file! { for ["test"] in "./programs/good" as validate_succeed => validate }
    test_each_file! { for ["test"] in "./programs/fail/validate" as validate_fail => validate }
}
