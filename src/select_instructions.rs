#![allow(unused)]

use crate::cvar::{CVarProgram, Tail};
use crate::lvar::{Expr, Op};
use crate::x86var::{Arg, Block, Cmd, Instr, Reg, X86VarProgram};

pub fn select_program(program: CVarProgram) -> X86VarProgram {
    X86VarProgram {
        blocks: program
            .blocks
            .into_iter()
            .map(|(sym, tail)| (sym, select_block(tail)))
            .collect(),
    }
}

fn select_block(tail: Tail) -> Block {
    let mut instrs = Vec::new();
    select_tail(tail, &mut instrs);
    Block { instrs }
}

fn select_tail(tail: Tail, instrs: &mut Vec<Instr>) {
    match tail {
        Tail::Return { expr } => instrs.extend(select_assign(String::default(), expr, true)),
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(sym, bnd, false));
            select_tail(*tail, &mut Vec::new());
        }
    }
}

fn select_assign(sym: String, expr: Expr, ret: bool) -> Vec<Instr> {
    let dst = if ret {
        Arg::Reg { reg: Reg::RAX }
    } else {
        Arg::XVar { sym }
    };

    match expr {
        // movq $val %dst
        Expr::Int { val } => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![Arg::Imm { val }, dst],
        }],

        // movq %sym %dst
        Expr::Var { sym } => vec![Instr::Instr {
            cmd: Cmd::Movq,
            args: vec![Arg::XVar { sym }, dst],
        }],

        // movq ?arg.0 %dst
        // addq ?arg.1 %dst
        Expr::Prim { op: Op::Plus, args } if args.len() == 2 => vec![
            Instr::Instr {
                cmd: Cmd::Movq,
                args: vec![select_atom(&args[0]), dst.clone()],
            },
            Instr::Instr {
                cmd: Cmd::Addq,
                args: vec![select_atom(&args[1]), dst],
            },
        ],

        // movq ?arg.0 %dst
        // subq ?arg.1 %dst
        Expr::Prim {
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
        Expr::Prim {
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
        Expr::Prim { op: Op::Read, args } if args.is_empty() => vec![
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
        Expr::Prim {
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

        Expr::Prim { .. } => {
            unreachable!("Encountered Prim with incorrect arity during select instructions pass.")
        }

        Expr::Let { .. } => {
            unreachable!("Encountered let-binding during select instructions pass.")
        }
    }
}

fn select_atom(expr: &Expr) -> Arg {
    match expr {
        Expr::Int { val } => Arg::Imm { val: *val },
        Expr::Var { sym } => Arg::XVar { sym: sym.clone() },
        Expr::Prim { .. } | Expr::Let { .. } => unreachable!("Tried to select a non-atom."),
    }
}
