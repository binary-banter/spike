use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::utils::split_test::split_test;
use derive_name::VariantName;
use miette::{NamedSource, Report};
use test_each_file::test_each_file;

fn validate([test]: [&str; 1], good: bool) {
    let (input, expected_output, expected_return, expected_error) = split_test(test);
    assert_eq!(good, expected_error.is_none());

    let result = parse_program(test).unwrap().validate();

    match (result, expected_error) {
        (Ok(program), None) => {
            let mut io = TestIO::new(input);
            let result = program.interpret(&mut io);

            assert_eq!(result, expected_return.into(), "Incorrect program result.");
            assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
        }
        (Ok(_), Some(expected_error)) => {
            panic!("Expected validation to fail with: {expected_error}.")
        }
        (Err(error), None) => {
            let report = Report::with_source_code(
                error.into(),
                NamedSource::new("<test file>", test.to_string()),
            );
            println!("{report}");
            panic!("Expected validation to succeed.")
        }
        (Err(error), Some(expected_error)) => {
            assert_eq!(error.variant_name(), expected_error);
        }
    }
}

test_each_file! { for ["test"] in "./programs/good" as validate_succeed => |i| validate(i, true) }
test_each_file! { for ["test"] in "./programs/fail/validate" as validate_fail => |i| validate(i, false) }
