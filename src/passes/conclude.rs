// //! This pass compiles `PX86Program`s  into `X86Program`.
// //!
// //! This pass generates the entry and exit point for the program wrapped around the body of the `PX86Program` program.
// //! Note that we will refer to the body of the `PX86Program` program as the 'core' block.
//
// use crate::language::x86var::{Block, PX86Program, Reg, X86Program};
// use crate::{addq, block, callq, imm, jmp, movq, popq, pushq, reg, subq};
//
// impl<'p> PX86Program<'p> {
//     /// See module-level documentation.
//     pub fn conclude(mut self) -> X86Program<'p> {
//         // main(&mut self);
//         // core(&mut self);
//         // conclusion(&mut self);
//         todo!();
//
//         X86Program {
//             blocks: self.blocks,
//             entry: self.entry,
//         }
//     }
// }
//
// // fn core(program: &mut PX86Program) {
// //     program
// //         .blocks
// //         .get_mut("core")
// //         .expect("There should be a core block.")
// //         .instrs
// //         .extend([jmp!("conclusion")]);
// // }
// //
// // fn main(program: &mut PX86Program) {
// //     program.blocks.insert(
// //         "main",
// //         block!(
// //             pushq!(reg!(RBP)),
// //             movq!(reg!(RSP), reg!(RBP)),
// //             subq!(imm!(program.stack_space as i64), reg!(RSP)),
// //             jmp!("core")
// //         ),
// //     );
// // }
// //
// // fn conclusion(program: &mut PX86Program) {
// //     program.blocks.insert(
// //         "conclusion",
// //         block!(
// //             addq!(imm!(program.stack_space as i64), reg!(RSP)),
// //             popq!(reg!(RBP)),
// //             movq!(reg!(RAX), reg!(RDI)),
// //             callq!("exit", 1)
// //         ),
// //     );
// // }
//
// #[cfg(test)]
// mod tests {
//     use crate::interpreter::TestIO;
//     use crate::language::x86var::X86VarProgram;
//     use crate::utils::split_test::split_test;
//     use test_each_file::test_each_file;
//
//     fn conclude([test]: [&str; 1]) {
//         let (input, expected_output, expected_return, program) = split_test(test);
//         let expected_return = expected_return.into();
//
//         let program: X86VarProgram = program
//             .uniquify()
//             .remove_complex_operands()
//             .explicate()
//             .select()
//             .add_liveness()
//             .compute_interference()
//             .color_interference()
//             .assign_homes()
//             .patch()
//             .conclude()
//             .into();
//         let mut io = TestIO::new(input);
//         let result = program.interpret("main", &mut io);
//
//         assert_eq!(result, expected_return, "Incorrect program result.");
//         assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
//     }
//
//     test_each_file! { for ["test"] in "./programs/good" as conclude => conclude }
// }
