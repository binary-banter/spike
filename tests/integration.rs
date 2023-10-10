use rust_compiler_construction::elf::ElfFile;
use rust_compiler_construction::utils::split_test::split_test;
use std::fs::File;
use std::io::{BufRead, Write};

use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tempdir::TempDir;
use test_each_file::test_each_file;
use rust_compiler_construction::interpreter::value::Val;

fn integration([test]: [&str; 1]) {
    let tempdir = TempDir::new("rust-compiler-construction-integration").unwrap();

    let mut file = File::create(tempdir.path().join("output")).unwrap();

    let (input, expected_output, expected_return, program) = split_test(test);
    let expected_return: i64 = expected_return.into();

    let (entry, program) = program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .add_liveness()
        .compute_interference()
        .color_interference()
        .assign_homes()
        .patch()
        .conclude()
        .emit();
    let elf = ElfFile::new(entry, &program);
    elf.write(&mut file);
    // file.set_permissions(Permissions::from_mode(0x777)).unwrap();
    file.flush().unwrap();

    drop(file);

    Command::new("chmod")
        .current_dir(&tempdir)
        .arg("+x")
        .arg("output")
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    sleep(Duration::from_secs(1));

    let mut program = Command::new("./output")
        .current_dir(&tempdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

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
        assert_eq!(got.parse::<Val>().unwrap(), expected);
    }
}

test_each_file! { for ["test"] in "./programs/good" as integration => integration }
