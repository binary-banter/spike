use crate::language::alvar::Atom;
use crate::language::cvar::CExpr;
use crate::language::cvar::{CVarProgram, Tail};
use crate::language::lvar::Op;
use crate::language::x86var::{Block, Cmd, Instr, Reg, VarArg, X86VarProgram};

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
        Tail::Return { expr } => instrs.extend(select_assign(String::default(), expr, true)),
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(sym, bnd, false));
            select_tail(*tail, instrs);
        }
    }
}

fn select_assign(sym: String, expr: CExpr, ret: bool) -> Vec<Instr<VarArg>> {
    let dst = if ret {
        VarArg::Reg { reg: Reg::RAX }
    } else {
        VarArg::XVar { sym }
    };

    match expr {
        // movq $val %dst
        CExpr::Atom(Atom::Int { val }) => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![VarArg::Imm { val }, dst],
        }],

        // movq %sym %dst
        CExpr::Atom(Atom::Var { sym }) => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![VarArg::XVar { sym }, dst],
        }],

        // movq ?arg.0 %dst
        // addq ?arg.1 %dst
        CExpr::Prim { op: Op::Plus, args } => match args.as_slice() {
            [arg0, arg1] => vec![
                Instr::Instr {
                    cmd: Cmd::Movq,
                    args: vec![select_atom(arg0), dst.clone()],
                },
                Instr::Instr {
                    cmd: Cmd::Addq,
                    args: vec![select_atom(arg1), dst],
                },
            ],
            _ => panic!("Addition is only defined for 2 arguments."),
        },

        // movq ?arg.0 %dst
        // subq ?arg.1 %dst
        CExpr::Prim {
            op: Op::Minus,
            args,
        } if args.len() == 2 => vec![
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![select_atom(&args[0]), dst.clone()],
            },
            Instr::Instr {
                cmd: Cmd::Subq,
                args: vec![select_atom(&args[1]), dst],
            },
        ],

        // movq ?arg.0 %dst
        // negq %dst
        CExpr::Prim {
            op: Op::Minus,
            args,
        } if args.len() == 1 => vec![
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![select_atom(&args[0]), dst.clone()],
            },
            Instr::Instr {
                cmd: Cmd::Negq,
                args: vec![dst],
            },
        ],

        // callq _read_int
        // movq  %rax %dst
        CExpr::Prim { op: Op::Read, args } if args.is_empty() => vec![
            Instr::Callq {
                lbl: "_read_int".to_string(),
                arity: 0,
            },
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![VarArg::Reg { reg: Reg::RAX }, dst],
            },
        ],

        // movq  %arg.0 %dst
        // movq  %arg.0 %RDI
        // callq _print_int
        CExpr::Prim {
            op: Op::Print,
            args,
        } if args.len() == 1 => vec![
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![select_atom(&args[0]), dst],
            },
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![select_atom(&args[0]), VarArg::Reg { reg: Reg::RDI }],
            },
            Instr::Callq {
                lbl: "_print_int".to_string(),
                arity: 1,
            },
        ],

        CExpr::Prim { .. } => {
            unreachable!("Encountered Prim with incorrect arity during select instructions pass.")
        }
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
