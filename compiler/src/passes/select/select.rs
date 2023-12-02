use crate::passes::atomize::Atom;
use crate::passes::eliminate::{ExprEliminated, TailEliminated, PrgEliminated, FunEliminated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Meta, UnaryOp};
use crate::passes::select::std_lib::add_std_library;
use crate::passes::select::{
    Block, Cnd, InstrSelected, VarArg, X86Selected, CALLEE_SAVED_NO_STACK, CALLER_SAVED,
};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::*;
use std::collections::HashMap;

impl<'p> PrgEliminated<'p> {
    #[must_use]
    pub fn select(self) -> X86Selected<'p> {
        let mut blocks = HashMap::new();
        for (_, fun) in self.fns {
            select_fun(fun, &mut blocks);
        }

        add_std_library(&self.std, &mut blocks);

        X86Selected {
            blocks,
            entry: self.entry,
            std: self.std,
        }
    }
}

fn select_fun<'p>(
    fun: FunEliminated<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Block<'p, VarArg<UniqueSym<'p>>>>,
) {
    // Collapse all exits from the function into one exit.
    let exit = gen_sym("fn_exit");
    let mut exit_instrs = Vec::new();
    for reg in CALLEE_SAVED_NO_STACK.into_iter().rev() {
        exit_instrs.push(popq!(VarArg::Reg { reg }));
    }
    exit_instrs.push(popq!(reg!(RBP)));
    exit_instrs.push(retq!());
    blocks.insert(exit, Block{ instrs: exit_instrs });

    for (block_sym, block) in fun.blocks {
        let mut instrs = Vec::new();

        if block_sym == fun.entry {
            instrs.push(pushq!(reg!(RBP)));
            instrs.push(movq!(reg!(RSP), reg!(RBP)));

            // TODO this is too much (sometimes)
            for reg in CALLEE_SAVED_NO_STACK {
                instrs.push(pushq!(VarArg::Reg { reg }));
            }

            for (reg, param) in CALLER_SAVED.into_iter().zip(fun.params.iter()) {
                instrs.push(movq!(VarArg::Reg { reg }, VarArg::XVar { sym: param.sym }));
            }
            assert!(
                fun.params.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );
        }

        select_tail(block, &mut instrs, exit);

        blocks.insert(block_sym, Block { instrs });
    }
}

fn select_tail<'p>(tail: TailEliminated<'p>, instrs: &mut Vec<InstrSelected<'p>>, exit: UniqueSym<'p>) {
    match tail {
        TailEliminated::Return { exprs } => {
            assert!(
                exprs.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );
            for (reg, arg) in CALLER_SAVED.into_iter().zip(exprs) {
                instrs.push(movq!(select_atom(arg), VarArg::Reg { reg }));
            }
            instrs.push(jmp!(exit));
        }
        TailEliminated::Seq {
            syms: sym,
            bnd,
            tail,
        } => {
            instrs.extend(select_assign(&sym, bnd));
            select_tail(*tail, instrs, exit);
        }
        TailEliminated::IfStmt { cnd, thn, els } => match cnd {
            ExprEliminated::BinaryOp {
                op,
                exprs: [expr_lhs, expr_rhs],
                ..
            } => {
                let tmp = gen_sym("tmp");
                instrs.extend(vec![
                    movq!(select_atom(expr_lhs), var!(tmp)),
                    cmpq!(select_atom(expr_rhs), var!(tmp)),
                    jcc!(thn, select_cmp(op)),
                    jmp!(els),
                ]);
            }
            _ => unreachable!(),
        },
        TailEliminated::Goto { lbl } => {
            instrs.push(jmp!(lbl));
        }
    }
}

fn select_assign<'p>(
    dsts: &[UniqueSym<'p>],
    expr: Meta<Vec<Type<UniqueSym<'p>>>, ExprEliminated<'p>>,
) -> Vec<InstrSelected<'p>> {
    let dst = var!(dsts[0]);
    match expr.inner {
        ExprEliminated::Atom {
            atm: Atom::Val { val },
            ..
        } => vec![movq!(imm!(val), dst)],
        ExprEliminated::Atom {
            atm: Atom::Var { sym },
            ..
        } => vec![movq!(var!(sym), dst)],
        ExprEliminated::BinaryOp {
            op,
            exprs: [a0, a1],
        } => match op {
            BinaryOp::Add => vec![
                movq!(select_atom(a0), dst.clone()),
                addq!(select_atom(a1), dst),
            ],
            BinaryOp::Sub => vec![
                movq!(select_atom(a0), dst.clone()),
                subq!(select_atom(a1), dst),
            ],
            BinaryOp::Mul => vec![
                movq!(select_atom(a1), reg!(RAX)),
                movq!(select_atom(a0), reg!(RBX)),
                mulq!(reg!(RBX)),
                movq!(reg!(RAX), dst),
            ],
            BinaryOp::Div => vec![
                movq!(imm!(0), reg!(RDX)),
                movq!(select_atom(a0), reg!(RAX)),
                movq!(select_atom(a1), reg!(RBX)),
                divq!(reg!(RBX)),
                movq!(reg!(RAX), dst),
            ],
            BinaryOp::Mod => vec![
                movq!(imm!(0), reg!(RDX)),
                movq!(select_atom(a0), reg!(RAX)),
                movq!(select_atom(a1), reg!(RBX)),
                divq!(reg!(RBX)),
                movq!(reg!(RDX), dst),
            ],
            BinaryOp::LAnd => vec![
                movq!(select_atom(a0), dst.clone()),
                andq!(select_atom(a1), dst),
            ],
            BinaryOp::LOr => vec![
                movq!(select_atom(a0), dst.clone()),
                orq!(select_atom(a1), dst),
            ],
            BinaryOp::Xor => vec![
                movq!(select_atom(a0), dst.clone()),
                xorq!(select_atom(a1), dst),
            ],
            op @ (BinaryOp::GT
            | BinaryOp::GE
            | BinaryOp::EQ
            | BinaryOp::LE
            | BinaryOp::LT
            | BinaryOp::NE) => {
                let tmp = gen_sym("tmp");
                vec![
                    movq!(select_atom(a0), var!(tmp)),
                    cmpq!(select_atom(a1), var!(tmp)),
                    movq!(imm!(0), reg!(RAX)),
                    setcc!(select_cmp(op)),
                    movq!(reg!(RAX), dst),
                ]
            }
        },
        ExprEliminated::UnaryOp { op, expr: a0 } => match op {
            UnaryOp::Neg => vec![movq!(select_atom(a0), dst.clone()), negq!(dst)],
            UnaryOp::Not => vec![movq!(select_atom(a0), dst.clone()), xorq!(imm!(1), dst)],
        },
        ExprEliminated::FunRef { sym, .. } => vec![load_lbl!(sym, dst)],
        ExprEliminated::Apply { fun, args, .. } => {
            let mut instrs = vec![];

            for (arg, reg) in args.iter().zip(CALLER_SAVED.into_iter()) {
                instrs.push(movq!(select_atom(*arg), VarArg::Reg { reg }));
            }
            assert!(
                args.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );

            instrs.push(callq_indirect!(select_atom(fun), args.len()));

            for (reg, dst) in CALLER_SAVED.into_iter().zip(dsts) {
                instrs.push(movq!(VarArg::Reg { reg }, var!(*dst)));
            }

            instrs
        }
        ExprEliminated::Asm { instrs } => instrs,
    }
}

fn select_atom(expr: Atom<'_>) -> VarArg<UniqueSym<'_>> {
    match expr {
        Atom::Val { val } => imm!(val),
        Atom::Var { sym } => var!(sym),
    }
}

fn select_cmp(op: BinaryOp) -> Cnd {
    match op {
        BinaryOp::GT => Cnd::GT,
        BinaryOp::GE => Cnd::GE,
        BinaryOp::EQ => Cnd::EQ,
        BinaryOp::LE => Cnd::LE,
        BinaryOp::LT => Cnd::LT,
        BinaryOp::NE => Cnd::NE,
        _ => unreachable!(),
    }
}
