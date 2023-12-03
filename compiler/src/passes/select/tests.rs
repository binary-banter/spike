use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::passes::select::{FunSelected, X86Selected};
use crate::utils::gen_sym::gen_sym;
use crate::utils::split_test::split_test;
use crate::*;
use std::collections::HashMap;
use test_each_file::test_each_file;
use crate::utils::test_runtime::add_runtime;

fn select([test]: [&str; 1]) {
    let (input, expected_output, expected_return, _) = split_test(test);

    let mut program = parse_program(test)
        .unwrap()
        .validate()
        .unwrap()
        .reveal()
        .atomize()
        .explicate()
        .eliminate()
        .select();

    add_runtime(&mut program);

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io);

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

test_each_file! { for ["sp"] in "./programs/good" as select => select }
