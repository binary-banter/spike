use crate::passes::atomize::Atom;
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
use crate::passes::parse::Op;
use crate::passes::select::io::Std;
use crate::passes::select::{
    Block, Cnd, Instr, VarArg, X86Selected, ARG_PASSING_REGS, CALLEE_SAVED_NO_STACK,
};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use crate::*;
use std::collections::HashMap;

impl<'p> PrgExplicated<'p> {
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
    tail: Tail<'p>,
    std: &Std<'p>,
    fn_params: &HashMap<UniqueSym<'p>, Vec<UniqueSym<'p>>>,
) -> Block<'p, VarArg<'p>> {
    let mut instrs = Vec::new();

    if let Some(params) = fn_params.get(&sym) {
        instrs.push(pushq!(reg!(RBP)));
        instrs.push(movq!(reg!(RSP), reg!(RBP)));
        for reg in CALLEE_SAVED_NO_STACK {
            instrs.push(pushq!(VarArg::Reg { reg }));
        }

        for (reg, param) in ARG_PASSING_REGS.into_iter().zip(params.iter()) {
            instrs.push(movq!(VarArg::Reg { reg }, VarArg::XVar { sym: *param }));
        }
        assert!(
            params.len() <= 6,
            "Argument passing to stack is not yet implemented."
        );
    }

    select_tail(tail, &mut instrs, std);

    Block { instrs }
}

fn select_tail<'p>(tail: Tail<'p>, instrs: &mut Vec<Instr<'p, VarArg<'p>>>, std: &Std<'p>) {
    match tail {
        Tail::Return { expr } => {
            instrs.extend(select_assign(reg!(RAX), expr, std));

            for reg in CALLEE_SAVED_NO_STACK.into_iter().rev() {
                instrs.push(popq!(VarArg::Reg { reg }));
            }
            instrs.push(popq!(reg!(RBP)));

            instrs.push(retq!());
        }
        Tail::Seq { sym, bnd, tail } => {
            instrs.extend(select_assign(var!(sym), bnd, std));
            select_tail(*tail, instrs, std);
        }
        Tail::IfStmt { cnd, thn, els } => match cnd {
            CExpr::Prim { op, args } => {
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
        CExpr::Atom {
            atm: Atom::Val { val },
        } => vec![movq!(imm!(val), dst)],
        CExpr::Atom {
            atm: Atom::Var { sym },
        } => vec![movq!(var!(sym), dst)],
        CExpr::Prim { op, args } => match (op, args.as_slice()) {
            (Op::Plus, [a0, a1]) => vec![movq!(select_atom(a0), dst), addq!(select_atom(a1), dst)],
            (Op::Minus, [a0, a1]) => vec![movq!(select_atom(a0), dst), subq!(select_atom(a1), dst)],
            (Op::Minus, [a0]) => vec![movq!(select_atom(a0), dst), negq!(dst)],
            (Op::Mul, [a0, a1]) => vec![
                movq!(select_atom(a1), reg!(RAX)),
                movq!(select_atom(a0), reg!(RBX)),
                mulq!(reg!(RBX)),
                movq!(reg!(RAX), dst),
            ],
            (Op::Div, [a0, a1]) => vec![
                movq!(imm!(0), reg!(RDX)),
                movq!(select_atom(a0), reg!(RAX)),
                movq!(select_atom(a1), reg!(RBX)),
                divq!(reg!(RBX)),
                movq!(reg!(RAX), dst),
            ],
            (Op::Mod, [a0, a1]) => vec![
                movq!(imm!(0), reg!(RDX)),
                movq!(select_atom(a0), reg!(RAX)),
                movq!(select_atom(a1), reg!(RBX)),
                divq!(reg!(RBX)),
                movq!(reg!(RDX), dst),
            ],
            (Op::Read, []) => {
                vec![callq_direct!(std.read_int, 0), movq!(reg!(RAX), dst)]
            }
            (Op::Print, [a0]) => vec![
                movq!(select_atom(a0), reg!(RDI)),
                callq_direct!(std.print_int, 1),
                movq!(select_atom(a0), dst),
            ],
            (Op::LAnd, [a0, a1]) => vec![movq!(select_atom(a0), dst), andq!(select_atom(a1), dst)],
            (Op::LOr, [a0, a1]) => vec![movq!(select_atom(a0), dst), orq!(select_atom(a1), dst)],
            (Op::Not, [a0]) => vec![movq!(select_atom(a0), dst), xorq!(imm!(1), dst)],
            (Op::Xor, [a0, a1]) => vec![movq!(select_atom(a0), dst), xorq!(select_atom(a1), dst)],
            (op @ (Op::GT | Op::GE | Op::EQ | Op::LE | Op::LT | Op::NE), [a0, a1]) => {
                let tmp = gen_sym("tmp");
                vec![
                    movq!(select_atom(a0), var!(tmp)),
                    cmpq!(select_atom(a1), var!(tmp)),
                    movq!(imm!(0), reg!(RAX)),
                    setcc!(select_cmp(op)),
                    movq!(reg!(RAX), dst),
                ]
            }
            _ => panic!("Encountered Prim with incorrect arity during select instructions pass."),
        },
        CExpr::FunRef { sym } => vec![load_lbl!(sym, dst)],
        CExpr::Apply { fun, args } => {
            let mut instrs = vec![];

            for (arg, reg) in args.iter().zip(ARG_PASSING_REGS.into_iter()) {
                instrs.push(movq!(select_atom(arg), VarArg::Reg { reg }));
            }
            assert!(
                args.len() <= 6,
                "Argument passing to stack is not yet implemented."
            );

            instrs.push(callq_indirect!(select_atom(&fun), args.len()));
            instrs.push(movq!(reg!(RAX), dst));
            instrs
        }
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
        Op::GT => Cnd::GT,
        Op::GE => Cnd::GE,
        Op::EQ => Cnd::EQ,
        Op::LE => Cnd::LE,
        Op::LT => Cnd::LT,
        Op::NE => Cnd::NE,
        _ => unreachable!(),
    }
}