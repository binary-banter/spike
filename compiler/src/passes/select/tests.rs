use crate::interpreter::TestIO;
use crate::passes::parse::parse::parse_program;
use crate::passes::select::{FunSelected, X86Selected};
use crate::utils::gen_sym::gen_sym;
use crate::utils::split_test::split_test;
use crate::*;
use std::collections::HashMap;
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

    add_runtime(&mut program);

    let mut io = TestIO::new(input);
    let result = program.interpret(&mut io);

    assert_eq!(result, expected_return.into(), "Incorrect program result.");
    assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
}

/// Adds runtime for testing.
pub fn add_runtime(program: &mut X86Selected) {
    let runtime = gen_sym("runtime");
    let entry = gen_sym("entry");
    let exit = gen_sym("exit");

    let entry_block = block!(callq_direct!(program.entry, 0), jmp!(exit));
    let exit_block = block!(
        movq!(reg!(RAX), reg!(RDI)),
        movq!(imm!(0x3C), reg!(RAX)),
        syscall!(2)
    );

    let runtime_fn = FunSelected {
        blocks: HashMap::from([(entry, entry_block), (exit, exit_block)]),
        entry,
        exit,
    };

    program.fns.insert(runtime, runtime_fn);
    program.entry = runtime;
}

test_each_file! { for ["sp"] in "./programs/good" as select => select }
