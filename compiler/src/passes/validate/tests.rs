use crate::passes::parse::parse::parse_program;
use crate::passes::validate::error::TypeError;
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;
use crate::passes::validate::PrgValidated;

fn validate([test]: [&str; 1]) {
    let (_, _, _, expected_error) = split_test(test);

    let result = parse_program(test).unwrap().validate();

    match (result, expected_error) {
        (Ok(_), None)=> {}
        (Ok(_), Some(expected_error))=> panic!("Expected validation to fail with: {expected_error}."),
        (Err(error), None)=> panic!("Expected validation to succeed but got: {error}"),
        (Err(_), Some(_)) => {}
    }
}

test_each_file! { for ["test"] in "./programs/good" as validate_succeed => validate }
test_each_file! { for ["test"] in "./programs/fail/validate" as validate_fail => validate }
