use crate::interpreter::value::Val;
use crate::language::lvar::{Lit, PrgParsed};
use crate::parser::parse_program;
use std::hash::Hash;

pub fn split_test_raw(test: &str) -> (Vec<Lit>, Vec<Lit>, Lit, &str) {
    let mut test = test.split('#');
    let input = test.next().unwrap().trim();
    let expected_output = test.next().unwrap().trim();

    let expected_return = test.next().unwrap().trim();

    let program = test.next().unwrap().trim();

    let input = input
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let expected_output = expected_output
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let expected_return = expected_return.parse().unwrap();

    (input, expected_output, expected_return, program)
}

pub fn split_test(test: &str) -> (Vec<Lit>, Vec<Lit>, Lit, PrgParsed) {
    let (input, expected_output, expected_return, program) = split_test_raw(test);
    let program = parse_program(program).unwrap();
    (input, expected_output, expected_return, program)
}
