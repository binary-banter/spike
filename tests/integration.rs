use rust_compiler_construction::passes::assign_homes::assign_program;
use rust_compiler_construction::passes::conclude::conclude_program;
use rust_compiler_construction::passes::emit::emit_program;
use rust_compiler_construction::passes::explicate_control::explicate_program;
use rust_compiler_construction::passes::patch_instructions::patch_program;
use rust_compiler_construction::passes::remove_complex_operands::rco_program;
use rust_compiler_construction::passes::select_instructions::select_program;
use rust_compiler_construction::passes::uniquify::uniquify_program;
use rust_compiler_construction::utils::split_test::split_test;
use std::fs::File;
use std::io::{BufRead, Write};
use std::process::{Command, Stdio};
use tempdir::TempDir;
use test_each_file::test_each_file;

fn integration([test]: [&str; 1]) {
    let tempdir = TempDir::new("rust-compiler-construction-integration").unwrap();

    let mut asm = File::create(tempdir.path().join("output.s")).unwrap();

    let (input, expected_output, expected_return, program) = split_test(test);
    emit_program(
        conclude_program(patch_program(assign_program(select_program(
            explicate_program(rco_program(uniquify_program(program))),
        )))),
        &mut asm,
    )
    .unwrap();

    Command::new("gcc")
        .current_dir(&tempdir)
        .arg("output.s")
        .arg("-no-pie")
        .arg("-Wa,-msyntax=intel")
        .args(["-o", "output"])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    let mut program = Command::new("./output")
        .current_dir(&tempdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = program.stdin.take().unwrap();

    for num in input {
        writeln!(stdin, "{num}\n").unwrap();
    }

    let out = program.wait_with_output().unwrap();
    assert_eq!(
        out.status.code().unwrap() as i64 & 0xFF,
        expected_return & 0xFF
    );
    for (got, expected) in out.stdout.lines().map(|r| r.unwrap()).zip(expected_output) {
        assert_eq!(got.parse::<i64>().unwrap(), expected);
    }
}

test_each_file! { for ["test"] in "./programs/good" as integration => integration }
