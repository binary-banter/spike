use miette::{NamedSource, Report};
use crate::passes::parse::parse::parse_program;
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;

fn validate([test]: [&str; 1]) {
    let (_, _, _, expected_error) = split_test(test);

    let result = parse_program(test).unwrap().validate();

    match (result, expected_error) {
        (Ok(_), None) => {}
        (Ok(_), Some(expected_error)) => {
            panic!("Expected validation to fail with: {expected_error}.")
        }
        (Err(error), None) => {
            dbg!(&error);
            let report = Report::with_source_code(error.into(), NamedSource::new("<test file>", test.to_string()));
            println!("{report}");
            panic!("Expected validation to succeed.")
        },
        (Err(_), Some(_)) => {}
    }
}

test_each_file! { for ["test"] in "./programs/good" as validate_succeed => validate }
test_each_file! { for ["test"] in "./programs/fail/validate" as validate_fail => validate }
