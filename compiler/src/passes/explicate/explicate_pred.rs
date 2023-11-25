use crate::passes::atomize::{AExpr, Atom};
use crate::passes::explicate::explicate::Env;
use crate::passes::explicate::explicate_assign::explicate_assign;
use crate::passes::explicate::{CExpr, CTail};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Meta, UnaryOp};
use crate::passes::validate::TLit;
use crate::utils::gen_sym::gen_sym;

pub fn explicate_pred<'p>(
    cnd: AExpr<'p>,
    thn: CTail<'p>,
    els: CTail<'p>,
    env: &mut Env<'_, 'p>,
) -> CTail<'p> {
    let mut create_block = |goto: CTail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
        sym
    };

    match cnd {
        AExpr::Atom {
            atm: Atom::Var { sym },
            ..
        } => CTail::IfStmt {
            cnd: CExpr::BinaryOp {
                op: BinaryOp::EQ,
                exprs: [
                    Atom::Var { sym },
                    Atom::Val {
                        val: TLit::Bool { val: true },
                    },
                ],
            },
            thn: create_block(thn),
            els: create_block(els),
        },
        AExpr::Atom {
            atm: Atom::Val {
                val: TLit::Bool { val },
            },
            ..
        } => {
            if val {
                thn
            } else {
                els
            }
        }
        AExpr::BinaryOp { op, exprs } => match op {
            BinaryOp::LAnd | BinaryOp::LOr | BinaryOp::Xor => {
                let tmp = gen_sym("tmp");
                explicate_assign(
                    tmp,
                    Meta {
                        meta: Type::Bool,
                        inner: AExpr::BinaryOp { op, exprs },
                    },
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
            BinaryOp::GT
            | BinaryOp::GE
            | BinaryOp::EQ
            | BinaryOp::LE
            | BinaryOp::LT
            | BinaryOp::NE => CTail::IfStmt {
                cnd: CExpr::BinaryOp { op, exprs },
                thn: create_block(thn),
                els: create_block(els),
            },
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                unreachable!("Unexpected `BinaryOp` in predicate position.")
            }
        },
        AExpr::UnaryOp { op, expr } => match op {
            UnaryOp::Not => explicate_pred(AExpr::Atom { atm: expr }, els, thn, env),
            UnaryOp::Neg => unreachable!("Unexpected `UnaryOp` in predicate position."),
        },
        AExpr::Let { sym, bnd, bdy, .. } => {
            explicate_assign(sym, *bnd, explicate_pred(bdy.inner, thn, els, env), env)
        }
        AExpr::If {
            cnd: cnd_sub,
            thn: thn_sub,
            els: els_sub,
            ..
        } => {
            let thn = create_block(thn);
            let els = create_block(els);

            explicate_pred(
                cnd_sub.inner,
                explicate_pred(
                    thn_sub.inner,
                    CTail::Goto { lbl: thn },
                    CTail::Goto { lbl: els },
                    env,
                ),
                explicate_pred(
                    els_sub.inner,
                    CTail::Goto { lbl: thn },
                    CTail::Goto { lbl: els },
                    env,
                ),
                env,
            )
        }
        AExpr::Apply { fun, args, .. } => {
            let tmp = gen_sym("tmp");
            explicate_assign(
                tmp,
                Meta {
                    meta: Type::Bool,
                    inner: AExpr::Apply { fun, args },
                },
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
            explicate_assign(
                tmp,
                Meta {
                    meta: Type::Bool,
                    inner: cnd,
                },
                explicate_pred(cnd_, thn, els, env),
                env,
            )
        }
        AExpr::Seq { stmt, cnt, .. } => explicate_assign(
            gen_sym("ignore"),
            *stmt,
            explicate_pred(cnt.inner, thn, els, env),
            env,
        ),
        AExpr::AccessField { strct, field, .. } => {
            let tmp = gen_sym("tmp");
            explicate_assign(
                tmp,
                Meta {
                    meta: Type::Bool,
                    inner: AExpr::AccessField { strct, field },
                },
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

        // cargo format should get some help
        AExpr::FunRef { .. }
        | AExpr::Atom {
            atm:
                Atom::Val {
                    val: TLit::U64 { .. } | TLit::I64 { .. } | TLit::Unit,
                },
            ..
        }
        | AExpr::Assign { .. }
        | AExpr::Break { .. }
        | AExpr::Continue { .. }
        | AExpr::Return { .. }
        | AExpr::Struct { .. } => unreachable!(),
    }
}
