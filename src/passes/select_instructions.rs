use crate::language::alvar::Atom;
use crate::language::cvar::{CExpr, CVarProgram, Tail};
use crate::language::lvar::Op;
use crate::language::x86var::{Arg, Block, Instr, Reg, VarArg, X86VarProgram};
use crate::{addq, callq, imm, movq, negq, reg, subq, var};

pub fn select_program(program: CVarProgram) -> X86VarProgram {
    X86VarProgram {
        blocks: vec![("start".to_string(), select_block(program.bdy))],
    }
}

fn select_block(tail: Tail) -> Block<VarArg> {
    let mut instrs = Vec::new();
    select_tail(tail, &mut instrs);
    Block { instrs }
}

fn select_tail(tail: Tail, instrs: &mut Vec<Instr<VarArg>>) {
    match tail {
        Tail::Return { expr } => instrs.extend(select_assign(reg!(RAX), expr)),
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(var!(sym), bnd));
            select_tail(*tail, instrs);
        }
    }
}

fn select_assign(dst: VarArg, expr: CExpr) -> Vec<Instr<VarArg>> {
    match expr {
        CExpr::Atom(Atom::Int { val }) => vec![movq!(imm!(val), dst)],
        CExpr::Atom(Atom::Var { sym }) => vec![movq!(var!(sym), dst)],
        CExpr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus, [a0, a1]) => vec![
                movq!(select_atom(a0), dst.clone()),
                addq!(select_atom(a1), dst),
            ],
            (Op::Minus, [a0, a1]) => vec![
                movq!(select_atom(a0), dst.clone()),
                subq!(select_atom(a1), dst),
            ],
            (Op::Minus, [a0]) => vec![movq!(select_atom(a0), dst.clone()), negq!(dst)],
            (Op::Read, []) => vec![callq!("_read_int", 0), movq!(reg!(RAX), dst)],
            (Op::Print, [a0]) => vec![
                movq!(select_atom(a0), dst),
                movq!(select_atom(a0), reg!(RDI)),
                callq!("_print_int", 1),
            ],
            _ => panic!("Encountered Prim with incorrect arity during select instructions pass."),
        },
    }
}

fn select_atom(expr: &Atom) -> VarArg {
    match expr {
        Atom::Int { val } => VarArg::Imm { val: *val },
        Atom::Var { sym } => VarArg::XVar { sym: sym.clone() },
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::x86var::interpret_x86var;
    use crate::interpreter::TestIO;
    use crate::passes::explicate_control::explicate_program;
    use crate::passes::remove_complex_operands::rco_program;
    use crate::passes::select_instructions::select_program;
    use crate::passes::uniquify::uniquify_program;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn select([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = select_program(explicate_program(rco_program(uniquify_program(program))));
        let mut io = TestIO::new(input);
        let result = interpret_x86var("start", &program, &mut io);

        assert_eq!(result, expected_return, "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as select_instructions => select }
}
