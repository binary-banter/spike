use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::passes::select::X86Selected;
use crate::utils::split_test::split_test;
use test_each_file::test_each_file;

fn conclude([test]: [&str; 1]) {
    let (input, expected_output, expected_return, _) = split_test(test);

    let program: X86Selected = parse_program(test)
        .unwrap()
        .validate()
        .unwrap()
        .reveal()
        .atomize()
        .explicate()
        .eliminate()
        .select()
        .add_liveness()
        .compute_interference()
        .color_interference()
        .assign_homes()
        .patch()
        .conclude()
        .into();

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io);

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

test_each_file! { for ["test"] in "./programs/good" as conclude => conclude }
