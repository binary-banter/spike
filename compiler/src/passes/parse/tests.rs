use crate::passes::parse::parse::{parse_program, PrettyParseError};
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;

fn parse([test]: [&str; 1]) {
    let (_, _, _, expected_error) = split_test(test);

    let result = parse_program(test);

    match (result, expected_error) {
        (Ok(_), None) => {}
        (Err(error), None) => {
            panic!("Should have succeeded, but panicked with `{error}` instead")
        }
        (Ok(_), Some(expected_error)) => {
            panic!("Expected error `{expected_error}`, but succeeded instead.")
        }
        (Err(error), Some(expected_error)) => match error {
            PrettyParseError::InvalidToken { .. } => {
                assert_eq!(expected_error, "InvalidToken")
            }
            PrettyParseError::UnexpectedToken { .. } => {
                assert_eq!(expected_error, "UnexpectedToken")
            }
            PrettyParseError::UnexpectedEOF { .. } => {
                assert_eq!(expected_error, "UnexpectedEOF")
            }
        },
    }
}

test_each_file! { for ["test"] in "./programs/good" as parse_succeed => parse }
test_each_file! { for ["test"] in "./programs/fail/parse" as parse_fail => parse }
