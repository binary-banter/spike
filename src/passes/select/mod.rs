//! This pass begins the work of translating from `CVarProgram` to `X86`.
//! The target language `X86VarProgram` of this pass is a variant of x86 that still uses variables.
//!
//! Just like a `CVarProgram` program, a `X86VarProgram` consists of a list of blocks.

pub mod io;

use std::collections::HashMap;
use crate::language::alvar::Atom;
use crate::language::cvar::{CExpr, CVarProgram, Tail};
use crate::language::lvar::Op;
use crate::language::x86var::{Block, Cnd, Instr, VarArg, X86VarProgram};
use crate::*;
use crate::passes::select::io::Std;

impl<'p> CVarProgram<'p> {
    /// See module-level documentation.
    pub fn select(self) -> X86VarProgram<'p> {
        let mut blocks = HashMap::new();
        let std = Std::new(&mut blocks);

        blocks.extend(
            self.blocks
                .into_iter()
                .map(|(name, block)| (name, select_block(block, &std))),
        );

        X86VarProgram {
            blocks,
            entry: self.entry,
            std
        }
    }
}

fn select_block<'p>(tail: Tail<'p>, std: &Std<'p>) -> Block<'p, VarArg<'p>> {
    let mut instrs = Vec::new();
    select_tail(tail, &mut instrs, std);
    Block { instrs }
}

fn select_tail<'p>(tail: Tail<'p>, instrs: &mut Vec<Instr<'p, VarArg<'p>>>, std: &Std<'p>) {
    match tail {
        Tail::Return { expr } => instrs.extend(select_assign(reg!(RAX), expr, std)),
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(var!(sym), bnd, std));
            select_tail(*tail, instrs, std);
        }
        Tail::IfStmt { cnd, thn, els } => match cnd {
            CExpr::Prim { op, args } => instrs.extend(vec![
                cmpq!(select_atom(&args[1]), select_atom(&args[0])),
                jcc!(thn, select_cmp(op)),
                jmp!(els),
            ]),
            _ => unreachable!(),
        },
        Tail::Goto { lbl } => {
            instrs.push(jmp!(lbl));
        }
    }
}

fn select_assign<'p>(
    dst: VarArg<'p>,
    expr: CExpr<'p>,
    std: &Std<'p>,
) -> Vec<Instr<'p, VarArg<'p>>> {
    match expr {
        CExpr::Atom(Atom::Val { val }) => vec![movq!(imm!(val), dst)],
        CExpr::Atom(Atom::Var { sym }) => vec![movq!(var!(sym), dst)],
        CExpr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus, [a0, a1]) => vec![movq!(select_atom(a0), dst), addq!(select_atom(a1), dst)],
            (Op::Minus, [a0, a1]) => vec![movq!(select_atom(a0), dst), subq!(select_atom(a1), dst)],
            (Op::Minus, [a0]) => vec![movq!(select_atom(a0), dst), negq!(dst)],
            (Op::Read, []) => {
                vec![callq!(std.read_int, 0), movq!(reg!(RAX), dst)]
            }
            (Op::Print, [a0]) => vec![
                movq!(select_atom(a0), reg!(RDI)),
                callq!(std.print_int, 1),
                movq!(select_atom(a0), dst),
            ],
            (Op::LAnd, [a0, a1]) => vec![movq!(select_atom(a0), dst), andq!(select_atom(a1), dst)],
            (Op::LOr, [a0, a1]) => vec![movq!(select_atom(a0), dst), orq!(select_atom(a1), dst)],
            (Op::Not, [a0]) => vec![movq!(select_atom(a0), dst), xorq!(imm!(1), dst)],
            (Op::Xor, [a0, a1]) => vec![movq!(select_atom(a0), dst), xorq!(select_atom(a1), dst)],
            (Op::Greater, [a0, a1]) => todo!(),
            (Op::GreaterOrEqual, [a0, a1]) => todo!(),
            (Op::Equal, [a0, a1]) => todo!(),
            (Op::LessOrEqual, [a0, a1]) => todo!(),
            (Op::Less, [a0, a1]) => todo!(),
            (Op::NotEqual, [a0, a1]) => todo!(),
            _ => panic!("Encountered Prim with incorrect arity during select instructions pass."),
        },
    }
}

fn select_atom<'p>(expr: &Atom<'p>) -> VarArg<'p> {
    match expr {
        Atom::Val { val } => imm!(*val),
        Atom::Var { sym } => var!(*sym),
    }
}

fn select_cmp(op: Op) -> Cnd {
    match op {
        Op::Greater => Cnd::Greater,
        Op::GreaterOrEqual => Cnd::GreaterOrEqual,
        Op::Equal => Cnd::Equal,
        Op::LessOrEqual => Cnd::LessOrEqual,
        Op::Less => Cnd::Less,
        Op::NotEqual => Cnd::NotEqual,
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;
    use crate::{callq, movq, reg};

    fn select([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let expected_return = expected_return.into();

        let mut program = program
            .uniquify()
            .remove_complex_operands()
            .explicate()
            .select();

        let entry = &mut program.blocks.get_mut(&program.entry).unwrap().instrs;
        entry.push(movq!(reg!(RAX), reg!(RDI)));
        entry.push(callq!(program.std.exit, 1));

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as select_instructions => select }
}
