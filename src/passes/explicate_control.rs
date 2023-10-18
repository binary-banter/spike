//! This pass compiles `ALVarProgram`s  into `CLVarProgram`.
//!
//! This pass makes the order of execution explicit in their syntax.
//! This is achieved by flattening the nested expressions into a sequence of statements.

use crate::language::alvar::ADef;
use crate::language::alvar::{AExpr, Atom, PrgAtomized};
use crate::language::cvar::{CExpr, PrgExplicated, Tail};
use crate::language::lvar::{Lit, Op};
use crate::passes::uniquify::{gen_sym, UniqueSym};
use std::collections::HashMap;

impl<'p> PrgAtomized<'p> {
    /// See module-level documentation.
    pub fn explicate(self) -> PrgExplicated<'p> {
        let mut blocks = HashMap::new();

        for (_, def) in self.defs {
            explicate_def(def, &mut blocks);
        }

        PrgExplicated {
            blocks,
            entry: self.entry,
        }
    }
}

fn explicate_def<'p>(def: ADef<'p>, blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>) {
    match def {
        ADef::Fn { sym, bdy, .. } => {
            let tail = explicate_tail(bdy, blocks);
            blocks.insert(sym, tail);
        }
    }
}

fn explicate_tail<'p>(expr: AExpr<'p>, blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>) -> Tail<'p> {
    match expr {
        AExpr::Atom { atm } => Tail::Return {
            expr: CExpr::Atom { atm },
        },
        AExpr::Prim { op, args } => Tail::Return {
            expr: CExpr::Prim { op, args },
        },
        AExpr::Let { sym, bnd, bdy } => {
            explicate_assign(sym, *bnd, explicate_tail(*bdy, blocks), blocks)
        }
        AExpr::If { cnd, thn, els } => explicate_pred(
            *cnd,
            explicate_tail(*thn, blocks),
            explicate_tail(*els, blocks),
            blocks,
        ),
        AExpr::Apply { .. } => todo!(),
        AExpr::FunRef { .. } => todo!(),
    }
}

fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: AExpr<'p>,
    tail: Tail<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("");
        blocks.insert(sym, goto);
        sym
    };

    match bnd {
        AExpr::Atom { atm } => Tail::Seq {
            sym,
            bnd: CExpr::Atom { atm },
            tail: Box::new(tail),
        },
        AExpr::Prim { op, args } => Tail::Seq {
            sym,
            bnd: CExpr::Prim { op, args },
            tail: Box::new(tail),
        },
        AExpr::Let {
            sym: sym_,
            bnd: bnd_,
            bdy: bdy_,
        } => explicate_assign(
            sym_,
            *bnd_,
            explicate_assign(sym, *bdy_, tail, blocks),
            blocks,
        ),
        AExpr::If { cnd, thn, els } => {
            let tb = create_block(tail);
            explicate_pred(
                *cnd,
                explicate_assign(sym, *thn, Tail::Goto { lbl: tb }, blocks),
                explicate_assign(sym, *els, Tail::Goto { lbl: tb }, blocks),
                blocks,
            )
        }
        AExpr::Apply { .. } => todo!(),
        AExpr::FunRef { .. } => todo!(),
    }
}

fn explicate_pred<'p>(
    cnd: AExpr<'p>,
    thn: Tail<'p>,
    els: Tail<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("");
        blocks.insert(sym, goto);
        sym
    };

    match cnd {
        AExpr::Atom {
            atm: Atom::Var { sym },
        } => Tail::IfStmt {
            cnd: CExpr::Prim {
                op: Op::EQ,
                args: vec![
                    Atom::Var { sym },
                    Atom::Val {
                        val: Lit::Bool { val: true },
                    },
                ],
            },
            thn: create_block(thn),
            els: create_block(els),
        },

        AExpr::Atom {
            atm: Atom::Val {
                val: Lit::Bool { val },
            },
        } => {
            if val {
                thn
            } else {
                els
            }
        }

        AExpr::Atom {
            atm: Atom::Val {
                val: Lit::Int { .. },
            },
        } => unreachable!(),

        AExpr::Prim { op: Op::Not, args } => match args.as_slice() {
            [atm] => explicate_pred(AExpr::Atom { atm: *atm }, els, thn, blocks),
            _ => unreachable!(),
        },

        AExpr::Prim {
            op: op @ (Op::EQ | Op::NE | Op::GT | Op::GE | Op::LT | Op::LE),
            args,
        } => Tail::IfStmt {
            cnd: CExpr::Prim { op, args },
            thn: create_block(thn),
            els: create_block(els),
        },

        AExpr::Prim { .. } => unreachable!(),

        AExpr::Let { sym, bnd, bdy } => {
            explicate_assign(sym, *bnd, explicate_pred(*bdy, thn, els, blocks), blocks)
        }

        AExpr::If {
            cnd: cnd_sub,
            thn: thn_sub,
            els: els_sub,
        } => {
            let thn = create_block(thn);
            let els = create_block(els);

            explicate_pred(
                *cnd_sub,
                explicate_pred(
                    *thn_sub,
                    Tail::Goto { lbl: thn },
                    Tail::Goto { lbl: els },
                    blocks,
                ),
                explicate_pred(
                    *els_sub,
                    Tail::Goto { lbl: thn },
                    Tail::Goto { lbl: els },
                    blocks,
                ),
                blocks,
            )
        }

        AExpr::Apply { .. } => todo!(),
        AExpr::FunRef { .. } => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn explicated([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .explicate();

        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as explicate => explicated }
}
