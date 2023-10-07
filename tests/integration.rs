// use rust_compiler_construction::utils::split_test::split_test;
// use std::fs::File;
// use std::io::{BufRead, Write};
// use std::process::{Command, Stdio};
// use tempdir::TempDir;
// use test_each_file::test_each_file;
//
// fn integration([test]: [&str; 1]) {
//     let tempdir = TempDir::new("rust-compiler-construction-integration").unwrap();
//
//     let mut asm = File::create(tempdir.path().join("output.s")).unwrap();
//
//     let (input, expected_output, expected_return, program) = split_test(test);
//     program
//         .uniquify()
//         .remove_complex_operands()
//         .explicate()
//         .select()
//         .assign_homes()
//         .patch()
//         .conclude()
//         .emit(&mut asm)
//         .unwrap();
//
//     Command::new("gcc")
//         .current_dir(&tempdir)
//         .arg("output.s")
//         .arg("-no-pie")
//         .arg("-Wa,-msyntax=intel")
//         .args(["-o", "output"])
//         .spawn()
//         .unwrap()
//         .wait()
//         .unwrap();
//
//     let mut program = Command::new("./output")
//         .current_dir(&tempdir)
//         .stdin(Stdio::piped())
//         .stdout(Stdio::piped())
//         .spawn()
//         .unwrap();
//
//     let mut stdin = program.stdin.take().unwrap();
//
//     for num in input {
//         writeln!(stdin, "{num}\n").unwrap();
//     }
//
//     let out = program.wait_with_output().unwrap();
//     assert_eq!(
//         out.status.code().unwrap() as i64 & 0xFF,
//         expected_return & 0xFF
//     );
//     for (got, expected) in out.stdout.lines().map(|r| r.unwrap()).zip(expected_output) {
//         assert_eq!(got.parse::<i64>().unwrap(), expected);
//     }
// }
//
// test_each_file! { for ["test"] in "./programs/good" as integration => integration }
