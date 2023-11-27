use crate::passes::atomize::Atom;
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Meta, Param, UnaryOp};
use crate::passes::select::std_lib::add_std_library;
use crate::passes::select::{
    Block, Cnd, Instr, VarArg, X86Selected, CALLEE_SAVED_NO_STACK, CALLER_SAVED,
};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::*;
use std::collections::HashMap;

impl<'p> PrgEliminated<'p> {
    #[must_use]
    pub fn select(self) -> X86Selected<'p> {
        let mut blocks = HashMap::new();

        blocks.extend(
            self.blocks
                .into_iter()
                .map(|(sym, block)| (sym, select_block(sym, block, &self.fn_params))),
        );

        add_std_library(&self.std, &mut blocks);

        X86Selected {
            blocks,
            entry: self.entry,
            // todo: technically we only need this for testing
            std: self.std,
        }
    }
}

fn select_block<'p>(
    sym: UniqueSym<'p>,
    tail: ETail<'p>,
    fn_params: &HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
) -> Block<'p, VarArg<UniqueSym<'p>>> {
    let mut instrs = Vec::new();

    if let Some(params) = fn_params.get(&sym) {
        instrs.push(pushq!(reg!(RBP)));
        instrs.push(movq!(reg!(RSP), reg!(RBP)));
        for reg in CALLEE_SAVED_NO_STACK {
            instrs.push(pushq!(VarArg::Reg { reg }));
        }

        for (reg, param) in CALLER_SAVED.into_iter().zip(params.iter()) {
            instrs.push(movq!(VarArg::Reg { reg }, VarArg::XVar { sym: param.sym }));
        }
        assert!(
            params.len() <= 9,
            "Argument passing to stack is not yet implemented."
        );
    }

    select_tail(tail, &mut instrs);

    Block { instrs }
}

fn select_tail<'p>(tail: ETail<'p>, instrs: &mut Vec<Instr<VarArg<UniqueSym<'p>>, UniqueSym<'p>>>) {
    match tail {
        ETail::Return { exprs } => {
            assert!(
                exprs.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );

            for (reg, arg) in CALLER_SAVED.into_iter().zip(exprs) {
                instrs.push(movq!(select_atom(arg), VarArg::Reg { reg }));
            }

            for reg in CALLEE_SAVED_NO_STACK.into_iter().rev() {
                instrs.push(popq!(VarArg::Reg { reg }));
            }
            instrs.push(popq!(reg!(RBP)));

            instrs.push(retq!());
        }
        ETail::Seq {
            syms: sym,
            bnd,
            tail,
        } => {
            instrs.extend(select_assign(&sym, bnd));
            select_tail(*tail, instrs);
        }
        ETail::IfStmt { cnd, thn, els } => match cnd {
            EExpr::BinaryOp {
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
        ETail::Goto { lbl } => {
            instrs.push(jmp!(lbl));
        }
    }
}

fn select_assign<'p>(
    dsts: &[UniqueSym<'p>],
    expr: Meta<Vec<Type<UniqueSym<'p>>>, EExpr<'p>>,
) -> Vec<Instr<VarArg<UniqueSym<'p>>, UniqueSym<'p>>> {
    let dst = var!(dsts[0]);
    match expr.inner {
        EExpr::Atom {
            atm: Atom::Val { val },
            ..
        } => vec![movq!(imm!(val), dst)],
        EExpr::Atom {
            atm: Atom::Var { sym },
            ..
        } => vec![movq!(var!(sym), dst)],
        EExpr::BinaryOp {
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
        EExpr::UnaryOp { op, expr: a0 } => match op {
            UnaryOp::Neg => vec![movq!(select_atom(a0), dst.clone()), negq!(dst)],
            UnaryOp::Not => vec![movq!(select_atom(a0), dst.clone()), xorq!(imm!(1), dst)],
        },
        EExpr::FunRef { sym, .. } => vec![load_lbl!(sym, dst)],
        EExpr::Apply { fun, args, .. } => {
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
        EExpr::Asm { instrs } => instrs,
    }
}

fn select_atom<'p>(expr: Atom<'p>) -> VarArg<UniqueSym<'p>> {
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
