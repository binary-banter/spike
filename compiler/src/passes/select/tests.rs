use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::utils::gen_sym::gen_sym;
use crate::utils::split_test::split_test;
use crate::{block, callq_direct, movq, reg};
use test_each_file::test_each_file;

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

    println!("{program}");

    // Redirect program to exit
    let new_entry = gen_sym("tmp");
    program.blocks.insert(
        new_entry,
        block!(
            callq_direct!(program.entry, 0),
            movq!(reg!(RAX), reg!(RDI)),
            callq_direct!(program.std["exit"], 1)
        ),
    );
    program.entry = new_entry;

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io);

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

test_each_file! { for ["test"] in "./programs/good" as select => select }
