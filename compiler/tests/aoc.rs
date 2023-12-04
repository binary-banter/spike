#![cfg(unix)]

use compiler::passes::parse::parse::parse_program;
use compiler::utils::split_test::split_test;
use std::fs;
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use tempfile::TempDir;
use test_each_file::{test_each_file, test_each_path};

fn aoc([test]: [&Path; 1]) {
    todo!()
    // let tempdir = TempDir::with_prefix("rust-compiler-construction-integration").unwrap();
    //
    // let (_, expected_output, _, _) = split_test(&fs::read_to_string(test).unwrap());
    // let expected_return: i64 = expected_return.into();
    //
    // let input_path = tempdir.path().join("output");
    // let mut output = OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .mode(0o777)
    //     .open(input_path)
    //     .unwrap();
    //
    // parse_program(test)
    //     .unwrap()
    //     .validate()
    //     .unwrap()
    //     .reveal()
    //     .atomize()
    //     .explicate()
    //     .eliminate()
    //     .select()
    //     .assign()
    //     .patch()
    //     .conclude()
    //     .emit()
    //     .write(&mut output);
    //
    // drop(output);
    //
    // // Wait for file to be readable
    // let mut program;
    // loop {
    //     let sub_res = Command::new("./output")
    //         .current_dir(&tempdir)
    //         .stdin(Stdio::piped())
    //         .stdout(Stdio::piped())
    //         .spawn();
    //     if let Err(e) = &sub_res {
    //         if e.kind().to_string() == *"executable file busy" {
    //             continue;
    //         }
    //     }
    //
    //     program = sub_res.unwrap();
    //     break;
    // }
    //
    // let mut stdin = program.stdin.take().unwrap();
    // for num in input {
    //     writeln!(stdin, "{num}").unwrap();
    // }
    //
    // let out = program.wait_with_output().unwrap();
    // assert_eq!(
    //     out.status.code().unwrap() as i64 & 0xFF,
    //     expected_return & 0xFF
    // );
    //
    // for (got, expected) in out.stdout.lines().map(|r| r.unwrap()).zip(expected_output) {
    //     assert_eq!(got.parse::<TLit>().unwrap(), expected);
    // }
}

test_each_path! { for ["sp, out"] in "./programs/aoc" as aoc => aoc }
