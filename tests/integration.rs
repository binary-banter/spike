use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::utils::split_test::split_test;
use std::fs::OpenOptions;
use std::io::{BufRead, Write};
use std::os::unix::prelude::OpenOptionsExt;

use rust_compiler_construction::language::lvar::Lit;
use std::process::{Command, Stdio};
use tempdir::TempDir;
use test_each_file::test_each_file;

fn integration([test]: [&str; 1]) {
    let tempdir = TempDir::new("rust-compiler-construction-integration").unwrap();

    let (input, expected_output, expected_return, program) = split_test(test);
    let expected_return: i64 = expected_return.into();

    let (entry, program) = program
        .type_check()
        .unwrap()
        .uniquify()
        .reveal()
        .atomize()
        .explicate()
        .select()
        .add_liveness()
        .compute_interference()
        .color_interference()
        .assign_homes()
        .patch()
        .conclude()
        .emit();

    let input_path = tempdir.path().join("output");
    let mut output = OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o777)
        .open(&input_path)
        .unwrap();

    let elf = ElfFile::new(entry, &program);
    elf.write(&mut output);
    drop(output);

    // Wait for file to be readable
    let mut program;
    loop {
        let sub_res = Command::new("./output")
            .current_dir(&tempdir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();
        if let Err(e) = &sub_res {
            if e.kind().to_string() == "executable file busy".to_string() {
                continue;
            }
        }

        program = sub_res.unwrap();
        break;
    }

    let mut stdin = program.stdin.take().unwrap();
    for num in input {
        writeln!(stdin, "{num}").unwrap();
    }

    let out = program.wait_with_output().unwrap();
    assert_eq!(
        out.status.code().unwrap() as i64 & 0xFF,
        expected_return & 0xFF
    );

    for (got, expected) in out.stdout.lines().map(|r| r.unwrap()).zip(expected_output) {
        assert_eq!(got.parse::<Lit>().unwrap(), expected);
    }
}

test_each_file! { for ["test"] in "./programs/good" as integration => integration }
