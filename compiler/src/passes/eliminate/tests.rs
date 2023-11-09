use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;

fn eliminate([test]: [&str; 1]) {
    let (input, expected_output, expected_return, _) = split_test(test);

    let program = parse_program(test)
        .unwrap()
        .validate()
        .unwrap()
        .reveal()
        .atomize()
        .explicate()
        .eliminate();

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io)[0];

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

test_each_file! { for ["test"] in "./programs/good" as eliminate => eliminate }
