//! This pass begins the work of translating from `CVarProgram` to `X86`.
//! The target language `X86VarProgram` of this pass is a variant of x86 that still uses variables.
//!
//! Just like a `CVarProgram` program, a `X86VarProgram` consists of a list of blocks.

use crate::language::alvar::Atom;
use crate::language::cvar::{CExpr, CVarProgram, Tail};
use crate::language::lvar::Op;
use crate::language::x86var::{Block, Instr, Reg, VarArg, X86VarProgram};
use crate::{addq, callq, imm, movq, negq, reg, subq, var};
use std::collections::HashMap;

impl<'p> CVarProgram<'p> {
    //! See module-level documentation.
    pub fn select(self) -> X86VarProgram<'p> {
        X86VarProgram {
            blocks: HashMap::from([("core", select_block(self.bdy))]),
        }
    }
}

fn select_block(tail: Tail<'_>) -> Block<'_, VarArg<'_>> {
    let mut instrs = Vec::new();
    select_tail(tail, &mut instrs);
    Block { instrs }
}

fn select_tail<'p>(tail: Tail<'p>, instrs: &mut Vec<Instr<'p, VarArg<'p>>>) {
    match tail {
        Tail::Return { expr } => instrs.extend(select_assign(reg!(RAX), expr)),
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(var!(sym), bnd));
            select_tail(*tail, instrs);
        }
    }
}

fn select_assign<'p>(dst: VarArg<'p>, expr: CExpr<'p>) -> Vec<Instr<'p, VarArg<'p>>> {
    match expr {
        CExpr::Atom(Atom::Int { val }) => vec![movq!(imm!(val), dst)],
        CExpr::Atom(Atom::Var { sym }) => vec![movq!(var!(sym), dst)],
        CExpr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus, [a0, a1]) => vec![movq!(select_atom(a0), dst), addq!(select_atom(a1), dst)],
            (Op::Minus, [a0, a1]) => vec![movq!(select_atom(a0), dst), subq!(select_atom(a1), dst)],
            (Op::Minus, [a0]) => vec![movq!(select_atom(a0), dst), negq!(dst)],
            (Op::Read, []) => vec![callq!("_read_int", 0), movq!(reg!(RAX), dst)],
            (Op::Print, [a0]) => vec![
                movq!(select_atom(a0), reg!(RDI)),
                callq!("_print_int", 1),
                movq!(select_atom(a0), dst),
            ],
            _ => panic!("Encountered Prim with incorrect arity during select instructions pass."),
        },
    }
}

fn select_atom<'p>(expr: &Atom<'p>) -> VarArg<'p> {
    match expr {
        Atom::Int { val } => VarArg::Imm { val: *val },
        Atom::Var { sym } => VarArg::XVar { sym: *sym },
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn select([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select();
        let mut io = TestIO::new(input);
        let result = program.interpret("core", &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as select_instructions => select }
}
