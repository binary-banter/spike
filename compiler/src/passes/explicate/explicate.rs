use crate::passes::atomize::{AExpr, Atom, PrgAtomized};
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
use crate::passes::parse::{Def, Lit, Op};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;

struct Env<'a, 'p> {
    blocks: &'a mut HashMap<UniqueSym<'p>, Tail<'p>>,
    /// (block to jump to, variable to write to)
    break_target: Option<(UniqueSym<'p>, UniqueSym<'p>)>,
    /// block to jump to
    continue_target: Option<UniqueSym<'p>>,
}

impl<'p> PrgAtomized<'p> {
    #[must_use]
    pub fn explicate(self) -> PrgExplicated<'p> {
        let mut blocks = HashMap::new();
        let mut env = Env {
            blocks: &mut blocks,
            break_target: None,
            continue_target: None,
        };

        let fn_params = self
            .defs
            .iter()
            .map(|(fn_sym, def)| match def {
                Def::Fn { params, .. } => (*fn_sym, params.iter().map(|param| param.sym).collect()),
                Def::Struct { .. } => todo!(),
                Def::Enum { .. } => todo!(),
            })
            .collect();

        for (_, def) in self.defs {
            explicate_def(def, &mut env);
        }

        PrgExplicated {
            blocks,
            fn_params,
            entry: self.entry,
        }
    }
}

fn explicate_def<'p>(def: Def<UniqueSym<'p>, AExpr<'p>>, env: &mut Env<'_, 'p>) {
    match def {
        Def::Fn { sym, bdy, .. } => {
            let tail = explicate_tail(bdy, env);
            env.blocks.insert(sym, tail);
        },
        Def::Struct { .. } => todo!(),
        Def::Enum { .. } => todo!(),
    }
}

fn explicate_tail<'p>(expr: AExpr<'p>, env: &mut Env<'_, 'p>) -> Tail<'p> {
    let tmp = gen_sym("return");
    let tail = Tail::Return {
        expr: CExpr::Atom {
            atm: Atom::Var { sym: tmp },
        },
    };
    explicate_assign(tmp, expr, tail, env)
}

fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: AExpr<'p>,
    tail: Tail<'p>,
    env: &mut Env<'_, 'p>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
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
        } => explicate_assign(sym_, *bnd_, explicate_assign(sym, *bdy_, tail, env), env),
        AExpr::If { cnd, thn, els } => {
            let tb = create_block(tail);
            explicate_pred(
                *cnd,
                explicate_assign(sym, *thn, Tail::Goto { lbl: tb }, env),
                explicate_assign(sym, *els, Tail::Goto { lbl: tb }, env),
                env,
            )
        }
        AExpr::Loop { bdy } => {
            let loop_block_sym = gen_sym("tmp");
            let tail = create_block(tail);
            let mut env = Env {
                blocks: env.blocks,
                break_target: Some((tail, sym)),
                continue_target: Some(loop_block_sym),
            };

            let loop_block = explicate_assign(
                gen_sym("ignore"),
                *bdy,
                Tail::Goto {
                    lbl: loop_block_sym,
                },
                &mut env,
            );
            env.blocks.insert(loop_block_sym, loop_block);
            Tail::Goto {
                lbl: loop_block_sym,
            }
        }
        AExpr::Break { bdy } => {
            let (break_sym, break_var) = env.break_target.unwrap();
            let break_goto = Tail::Goto { lbl: break_sym };
            explicate_assign(break_var, *bdy, break_goto, env)
        }
        AExpr::Seq { stmt, cnt } => explicate_assign(
            gen_sym("ignore"),
            *stmt,
            explicate_assign(sym, *cnt, tail, env),
            env,
        ),
        AExpr::Assign {
            sym: sym_,
            bnd: bnd_,
        } => explicate_assign(
            sym_,
            *bnd_,
            explicate_assign(
                sym,
                AExpr::Atom {
                    atm: Atom::Val { val: Lit::Unit },
                },
                tail,
                env,
            ),
            env,
        ),
        AExpr::Continue => Tail::Goto {
            lbl: env.continue_target.unwrap(),
        },
        AExpr::Return { bdy } => {
            let tmp = gen_sym("return");
            let tail = Tail::Return {
                expr: CExpr::Atom {
                    atm: Atom::Var { sym: tmp },
                },
            };
            explicate_assign(tmp, *bdy, tail, env)
        }
    }
}

fn explicate_pred<'p>(
    cnd: AExpr<'p>,
    thn: Tail<'p>,
    els: Tail<'p>,
    env: &mut Env<'_, 'p>,
) -> Tail<'p> {
    let mut create_block = |goto: Tail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
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
            Op::Not => explicate_pred(AExpr::Atom { atm: args[0] }, els, thn, env),
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
                        env,
                    ),
                    env,
                )
            }
            Op::Read | Op::Print | Op::Plus | Op::Minus | Op::Mul | Op::Mod | Op::Div => {
                unreachable!()
            }
        },
        AExpr::Let { sym, bnd, bdy } => {
            explicate_assign(sym, *bnd, explicate_pred(*bdy, thn, els, env), env)
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
                    env,
                ),
                explicate_pred(
                    *els_sub,
                    Tail::Goto { lbl: thn },
                    Tail::Goto { lbl: els },
                    env,
                ),
                env,
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
                    env,
                ),
                env,
            )
        }
        AExpr::Loop { .. } => {
            let tmp = gen_sym("tmp");
            let cnd_ = AExpr::Atom {
                atm: Atom::Var { sym: tmp },
            };
            explicate_assign(tmp, cnd, explicate_pred(cnd_, thn, els, env), env)
        }
        AExpr::Seq { stmt, cnt } => explicate_assign(
            gen_sym("ignore"),
            *stmt,
            explicate_pred(*cnt, thn, els, env),
            env,
        ),
        // cargo format should get some help
        AExpr::FunRef { .. }
        | AExpr::Atom {
            atm: Atom::Val {
                val: Lit::Int { .. } | Lit::Unit,
            },
        }
        | AExpr::Assign { .. }
        | AExpr::Break { .. }
        | AExpr::Continue
        | AExpr::Return { .. } => unreachable!(),
    }
}
