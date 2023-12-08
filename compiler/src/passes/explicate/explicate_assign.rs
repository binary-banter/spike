use crate::passes::atomize::{AExpr, Atom};
use crate::passes::explicate::explicate::Env;
use crate::passes::explicate::{explicate_pred, ExprExplicated, TailExplicated};

use crate::passes::parse::{Lit, Meta, Typed};
use crate::utils::unique_sym::{gen_sym, UniqueSym};

pub fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: Typed<'p, AExpr<'p>>,
    tail: TailExplicated<'p>,
    env: &mut Env<'_, 'p>,
) -> TailExplicated<'p> {
    let mut create_block = |goto: TailExplicated<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
        sym
    };

    match bnd.inner {
        AExpr::Apply { fun, args } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::Apply { fun, args },
            },
            tail: Box::new(tail),
        },
        AExpr::FunRef { sym: sym_fn } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::FunRef { sym: sym_fn },
            },
            tail: Box::new(tail),
        },
        AExpr::Atom { atm } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::Atom { atm },
            },
            tail: Box::new(tail),
        },
        AExpr::BinaryOp { op, exprs } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::BinaryOp { op, exprs },
            },
            tail: Box::new(tail),
        },
        AExpr::UnaryOp { op, expr } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::UnaryOp { op, expr },
            },
            tail: Box::new(tail),
        },
        AExpr::Let {
            sym: sym_,
            bnd: bnd_,
            bdy: bdy_,
            ..
        } => explicate_assign(sym_, *bnd_, explicate_assign(sym, *bdy_, tail, env), env),
        AExpr::If { cnd, thn, els, .. } => {
            let tb = create_block(tail);
            explicate_pred::explicate_pred(
                cnd.inner,
                explicate_assign(sym, *thn, TailExplicated::Goto { lbl: tb }, env),
                explicate_assign(sym, *els, TailExplicated::Goto { lbl: tb }, env),
                env,
            )
        }
        AExpr::Loop { bdy, .. } => {
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
                TailExplicated::Goto {
                    lbl: loop_block_sym,
                },
                &mut env,
            );
            env.blocks.insert(loop_block_sym, loop_block);
            TailExplicated::Goto {
                lbl: loop_block_sym,
            }
        }
        AExpr::Break { bdy, .. } => {
            let (break_sym, break_var) = env.break_target.unwrap();
            let break_goto = TailExplicated::Goto { lbl: break_sym };
            explicate_assign(break_var, *bdy, break_goto, env)
        }
        AExpr::Seq { stmt, cnt, .. } => explicate_assign(
            gen_sym("ignore"),
            *stmt,
            explicate_assign(sym, *cnt, tail, env),
            env,
        ),
        AExpr::Assign {
            sym: sym_,
            bnd: bnd_,
            ..
        } => explicate_assign(
            sym_,
            *bnd_,
            explicate_assign(
                sym,
                Meta {
                    meta: bnd.meta,
                    inner: AExpr::Atom {
                        atm: Atom::Val { val: Lit::Unit },
                    },
                },
                tail,
                env,
            ),
            env,
        ),
        AExpr::Continue { .. } => TailExplicated::Goto {
            lbl: env.continue_target.unwrap(),
        },
        AExpr::Return { bdy, .. } => {
            let tmp = gen_sym("return");
            let tail = TailExplicated::Return {
                expr: Meta {
                    meta: bnd.meta,
                    inner: Atom::Var { sym: tmp },
                },
            };
            explicate_assign(tmp, *bdy, tail, env)
        }
        AExpr::Struct { sym: sym_, fields } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::Struct { sym: sym_, fields },
            },
            tail: Box::new(tail),
        },
        AExpr::AccessField { strct, field } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::AccessField { strct, field },
            },
            tail: Box::new(tail),
        },
        AExpr::Asm { instrs } => TailExplicated::Seq {
            sym,
            bnd: Meta {
                meta: bnd.meta,
                inner: ExprExplicated::Asm { instrs },
            },
            tail: Box::new(tail),
        },
    }
}
