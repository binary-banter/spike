//! This pass compiles `ALVarProgram`s  into `CLVarProgram`.
//!
//! This pass makes the order of execution explicit in their syntax.
//! This is achieved by flattening the nested expressions into a sequence of statements.

use crate::language::cvar::{CExpr, PrgExplicated, Tail};
use crate::passes::parse::{Def, Lit, Op};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;
use crate::passes::atomize::{AExpr, Atom, PrgAtomized};

impl<'p> PrgAtomized<'p> {
    /// See module-level documentation.
    pub fn explicate(self) -> PrgExplicated<'p> {
        let mut blocks = HashMap::new();
        let fn_params = self
            .defs
            .iter()
            .map(|(fn_sym, def)| match def {
                Def::Fn { params, .. } => (*fn_sym, params.iter().map(|(sym, _)| *sym).collect()),
            })
            .collect();

        for (_, def) in self.defs {
            explicate_def(def, &mut blocks);
        }

        PrgExplicated {
            blocks,
            fn_params,
            entry: self.entry,
        }
    }
}

fn explicate_def<'p>(def: Def<UniqueSym<'p>, AExpr<'p>>, blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>) {
    match def {
        Def::Fn { sym, bdy, .. } => {
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
        AExpr::Apply { fun, args } => Tail::Return {
            expr: CExpr::Apply { fun, args },
        },
        AExpr::FunRef { sym } => Tail::Return {
            expr: CExpr::FunRef { sym },
        },
    }
}

fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: AExpr<'p>,
    tail: Tail<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("tmp");
        blocks.insert(sym, goto);
        sym
    };

    match bnd {
        AExpr::Apply { fun, args } => Tail::Seq {
            sym,
            bnd: CExpr::Apply { fun, args },
            tail: Box::new(tail),
        },
        AExpr::FunRef { sym: sym_fn } => Tail::Seq {
            sym,
            bnd: CExpr::FunRef { sym: sym_fn },
            tail: Box::new(tail),
        },
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
    }
}

fn explicate_pred<'p>(
    cnd: AExpr<'p>,
    thn: Tail<'p>,
    els: Tail<'p>,
    blocks: &mut HashMap<UniqueSym<'p>, Tail<'p>>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("tmp");
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

        AExpr::Prim { op, args } => match op {
            Op::Not => explicate_pred(AExpr::Atom { atm: args[0] }, els, thn, blocks),
            Op::EQ | Op::NE | Op::GT | Op::GE | Op::LT | Op::LE => Tail::IfStmt {
                cnd: CExpr::Prim { op, args },
                thn: create_block(thn),
                els: create_block(els),
            },
            Op::LAnd | Op::LOr | Op::Xor => {
                let tmp = gen_sym("tmp");
                explicate_assign(
                    tmp,
                    AExpr::Prim { op, args },
                    explicate_pred(
                        AExpr::Atom {
                            atm: Atom::Var { sym: tmp },
                        },
                        thn,
                        els,
                        blocks,
                    ),
                    blocks,
                )
            }

            Op::Read | Op::Print | Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div => {
                unreachable!()
            }
        },

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

        AExpr::Apply { fun, args } => {
            let tmp = gen_sym("tmp");
            explicate_assign(
                tmp,
                AExpr::Apply { fun, args },
                explicate_pred(
                    AExpr::Atom {
                        atm: Atom::Var { sym: tmp },
                    },
                    thn,
                    els,
                    blocks,
                ),
                blocks,
            )
        }

        AExpr::FunRef { .. }
        | AExpr::Atom {
            atm: Atom::Val {
                val: Lit::Int { .. },
            },
        } => unreachable!(),
        AExpr::Atom {
            atm: Atom::Val { val: Lit::Unit },
        } => todo!(),
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
