use crate::passes::atomize::Atom;
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::parse::{BinaryOp, Param};
use crate::passes::select::io::Std;
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
        let std = Std::new(&mut blocks);

        blocks.extend(
            self.blocks
                .into_iter()
                .map(|(sym, block)| (sym, select_block(sym, block, &std, &self.fn_params))),
        );

        X86Selected {
            blocks,
            entry: self.entry,
            std,
        }
    }
}

fn select_block<'p>(
    sym: UniqueSym<'p>,
    tail: ETail<'p>,
    std: &Std<'p>,
    fn_params: &HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
) -> Block<'p, VarArg<'p>> {
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

    select_tail(tail, &mut instrs, std);

    Block { instrs }
}

fn select_tail<'p>(tail: ETail<'p>, instrs: &mut Vec<Instr<'p, VarArg<'p>>>, std: &Std<'p>) {
    match tail {
        ETail::Return { exprs } => {
            assert!(
                exprs.len() <= 9,
                "Argument passing to stack is not yet implemented."
            );

            for (reg, (arg, _)) in CALLER_SAVED.into_iter().zip(exprs) {
                instrs.push(movq!(select_atom(&arg), VarArg::Reg { reg }));
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
            instrs.extend(select_assign(&sym, bnd, std));
            select_tail(*tail, instrs, std);
        }
        ETail::IfStmt { cnd, thn, els } => match cnd {
            EExpr::Prim { op, args, .. } => {
                let tmp = gen_sym("tmp");
                instrs.extend(vec![
                    movq!(select_atom(&args[0]), var!(tmp)),
                    cmpq!(select_atom(&args[1]), var!(tmp)),
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
    expr: EExpr<'p>,
    std: &Std<'p>,
) -> Vec<Instr<'p, VarArg<'p>>> {
    todo!()
    // let dst = var!(dsts[0]);
    // match expr {
    //     EExpr::Atom {
    //         atm: Atom::Val { val },
    //         ..
    //     } => vec![movq!(imm!(val), dst)],
    //     EExpr::Atom {
    //         atm: Atom::Var { sym },
    //         ..
    //     } => vec![movq!(var!(sym), dst)],
    //     EExpr::Prim { op, args, .. } => match (op, args.as_slice()) {
    //         (BinaryOp::Add, [a0, a1]) => vec![
    //             movq!(select_atom(a0), dst.clone()),
    //             addq!(select_atom(a1), dst),
    //         ],
    //         (BinaryOp::Sub, [a0, a1]) => vec![
    //             movq!(select_atom(a0), dst.clone()),
    //             subq!(select_atom(a1), dst),
    //         ],
    //         (BinaryOp::Minus, [a0]) => vec![movq!(select_atom(a0), dst.clone()), negq!(dst)],
    //         (BinaryOp::Mul, [a0, a1]) => vec![
    //             movq!(select_atom(a1), reg!(RAX)),
    //             movq!(select_atom(a0), reg!(RBX)),
    //             mulq!(reg!(RBX)),
    //             movq!(reg!(RAX), dst),
    //         ],
    //         (BinaryOp::Div, [a0, a1]) => vec![
    //             movq!(imm!(0), reg!(RDX)),
    //             movq!(select_atom(a0), reg!(RAX)),
    //             movq!(select_atom(a1), reg!(RBX)),
    //             divq!(reg!(RBX)),
    //             movq!(reg!(RAX), dst),
    //         ],
    //         (BinaryOp::Mod, [a0, a1]) => vec![
    //             movq!(imm!(0), reg!(RDX)),
    //             movq!(select_atom(a0), reg!(RAX)),
    //             movq!(select_atom(a1), reg!(RBX)),
    //             divq!(reg!(RBX)),
    //             movq!(reg!(RDX), dst),
    //         ],
    //         (BinaryOp::Read, []) => {
    //             vec![callq_direct!(std.read_int, 0), movq!(reg!(RAX), dst)]
    //         }
    //         (BinaryOp::Print, [a0]) => vec![
    //             movq!(select_atom(a0), reg!(RDI)),
    //             callq_direct!(std.print_int, 1),
    //             movq!(select_atom(a0), dst),
    //         ],
    //         (BinaryOp::LAnd, [a0, a1]) => vec![
    //             movq!(select_atom(a0), dst.clone()),
    //             andq!(select_atom(a1), dst),
    //         ],
    //         (BinaryOp::LOr, [a0, a1]) => vec![
    //             movq!(select_atom(a0), dst.clone()),
    //             orq!(select_atom(a1), dst),
    //         ],
    //         (BinaryOp::Not, [a0]) => vec![movq!(select_atom(a0), dst.clone()), xorq!(imm!(1), dst)],
    //         (BinaryOp::Xor, [a0, a1]) => vec![
    //             movq!(select_atom(a0), dst.clone()),
    //             xorq!(select_atom(a1), dst),
    //         ],
    //         (op @ (BinaryOp::GT | BinaryOp::GE | BinaryOp::EQ | BinaryOp::LE | BinaryOp::LT | BinaryOp::NE), [a0, a1]) => {
    //             let tmp = gen_sym("tmp");
    //             vec![
    //                 movq!(select_atom(a0), var!(tmp)),
    //                 cmpq!(select_atom(a1), var!(tmp)),
    //                 movq!(imm!(0), reg!(RAX)),
    //                 setcc!(select_cmp(op)),
    //                 movq!(reg!(RAX), dst),
    //             ]
    //         }
    //         _ => panic!("Encountered Prim with incorrect arity during select instructions pass."),
    //     },
    //     EExpr::FunRef { sym, .. } => vec![load_lbl!(sym, dst)],
    //     EExpr::Apply { fun, args, .. } => {
    //         let mut instrs = vec![];
    //
    //         for ((arg, _), reg) in args.iter().zip(CALLER_SAVED.into_iter()) {
    //             instrs.push(movq!(select_atom(arg), VarArg::Reg { reg }));
    //         }
    //         assert!(
    //             args.len() <= 9,
    //             "Argument passing to stack is not yet implemented."
    //         );
    //
    //         instrs.push(callq_indirect!(select_atom(&fun), args.len()));
    //
    //         for (reg, dst) in CALLER_SAVED.into_iter().zip(dsts) {
    //             instrs.push(movq!(VarArg::Reg { reg }, var!(*dst)));
    //         }
    //
    //         instrs
    //     }
    // }
}

fn select_atom<'p>(expr: &Atom<'p>) -> VarArg<'p> {
    match expr {
        Atom::Val { val } => imm!(*val),
        Atom::Var { sym } => var!(*sym),
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
