use rust_compiler_construction::utils::split_test::split_test;
use std::fs::{File, Permissions};
use std::io::{BufRead, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use tempdir::TempDir;
use test_each_file::test_each_file;
use rust_compiler_construction::elf::ElfFile;

fn integration([test]: [&str; 1]) {
    let tempdir = TempDir::new("rust-compiler-construction-integration").unwrap();

    let mut file = File::create(tempdir.path().join("output")).unwrap();

    let (input, expected_output, expected_return, program) = split_test(test);

    let (entry, program) = program
        .uniquify()
        .remove_complex_operands()
        .explicate()
        .select()
        .assign_homes()
        .patch()
        .conclude()
        .emit();
    let elf = ElfFile::new(entry, &program);
    elf.write(&mut file);
    // file.set_permissions(Permissions::from_mode(0x777)).unwrap();
    file.flush().unwrap();

    drop(file);

    Command::new("chmod").current_dir(&tempdir).arg("+x").arg("output").spawn().unwrap().wait().unwrap();

    sleep(Duration::from_secs(1));

    let mut program = Command::new("./output")
        .current_dir(&tempdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    //TODO
    // let mut stdin = program.stdin.take().unwrap();
    // for num in input {
    //     writeln!(stdin, "{num}\n").unwrap();
    // }

    let out = program.wait_with_output().unwrap();
    assert_eq!(
        out.status.code().unwrap() as i64 & 0xFF,
        expected_return & 0xFF
    );
    //TODO
    // for (got, expected) in out.stdout.lines().map(|r| r.unwrap()).zip(expected_output) {
    //     assert_eq!(got.parse::<i64>().unwrap(), expected);
    // }
}

test_each_file! { for ["test"] in "./programs/good" as integration => integration }
