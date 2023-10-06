use crate::language::lvar::LVarProgram;
use crate::parser::parse_program;

pub fn split_test(test: &str) -> (Vec<i64>, Vec<i64>, i64, LVarProgram) {
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
    let program = parse_program(program).unwrap();

    (input, expected_output, expected_return, program)
}
