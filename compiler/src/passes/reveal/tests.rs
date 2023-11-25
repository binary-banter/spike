use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::passes::validate::PrgValidated;
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;

fn reveal([test]: [&str; 1]) {
    let (input, expected_output, expected_return, _) = split_test(test);

    let program: PrgValidated = parse_program(test)
        .unwrap()
        .validate()
        .unwrap()
        .reveal()
        .into();

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io);

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

test_each_file! { for ["test"] in "./programs/good" as reveal => reveal }
