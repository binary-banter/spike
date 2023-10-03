use crate::language::alvar::Atom;
use crate::language::cvar::CExpr;
use crate::language::cvar::{CVarProgram, Tail};
use crate::language::lvar::Op;
use crate::x86var::{Arg, Block, Cmd, Instr, Reg, X86VarProgram};

pub fn select_program(program: CVarProgram) -> X86VarProgram {
    X86VarProgram {
        blocks: vec![("start".to_string(), select_block(program.bdy))],
    }
}

fn select_block(tail: Tail) -> Block {
    let mut instrs = Vec::new();
    select_tail(tail, &mut instrs);
    Block { instrs }
}

fn select_tail(tail: Tail, instrs: &mut Vec<Instr>) {
    match tail {
        Tail::Return { expr } => {
            instrs.extend(select_assign(String::default(), expr, true).into_iter())
        }
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(sym, bnd, false));
            select_tail(*tail, instrs);
        }
    }
}

fn select_assign(sym: String, expr: CExpr, ret: bool) -> Vec<Instr> {
    let dst = if ret {
        Arg::Reg { reg: Reg::RAX }
    } else {
        Arg::XVar { sym }
    };

    match expr {
        // movq $val %dst
        CExpr::Atom(Atom::Int { val }) => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![Arg::Imm { val }, dst],
        }],

        // movq %sym %dst
        CExpr::Atom(Atom::Var { sym }) => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![Arg::XVar { sym }, dst],
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
                args: vec![Arg::Reg { reg: Reg::RAX }, dst],
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
                args: vec![select_atom(&args[0]), Arg::Reg { reg: Reg::RDI }],
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

fn select_atom(expr: &Atom) -> Arg {
    match expr {
        Atom::Int { val } => Arg::Imm { val: *val },
        Atom::Var { sym } => Arg::XVar { sym: sym.clone() },
    }
}
