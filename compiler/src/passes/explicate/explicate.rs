use crate::passes::atomize::{AExpr, Atom, PrgAtomized};
use crate::passes::explicate::{CExpr, CTail, PrgExplicated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Op, TypeDef};
use crate::passes::validate::TLit;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;

struct Env<'a, 'p> {
    blocks: &'a mut HashMap<UniqueSym<'p>, CTail<'p>>,
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

        let mut fn_params = HashMap::new();

        for (sym, def) in &self.defs {
            match def {
                Def::Fn { params, .. } => {
                    fn_params.insert(*sym, params.clone());
                }
                Def::TypeDef { .. } => {
                    // todo?
                }
            }
        }

        let mut defs = HashMap::new();

        for (_, def) in self.defs {
            explicate_def(def, &mut env, &mut defs);
        }

        PrgExplicated {
            blocks,
            fn_params,
            defs,
            entry: self.entry,
        }
    }
}

fn explicate_def<'p>(
    def: Def<UniqueSym<'p>, &'p str, AExpr<'p>>,
    env: &mut Env<'_, 'p>,
    defs: &mut HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) {
    match def {
        Def::Fn { sym, bdy, .. } => {
            let tail = explicate_tail(bdy, env);
            env.blocks.insert(sym, tail);
        }
        Def::TypeDef { sym, def } => {
            defs.insert(sym, def);
        }
    }
}

fn explicate_tail<'p>(expr: AExpr<'p>, env: &mut Env<'_, 'p>) -> CTail<'p> {
    let tmp = gen_sym("return");
    let tail = CTail::Return {
        expr: Atom::Var { sym: tmp },
        typ: expr.typ().clone(),
    };
    explicate_assign(tmp, expr, tail, env)
}

fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: AExpr<'p>,
    tail: CTail<'p>,
    env: &mut Env<'_, 'p>,
) -> CTail<'p> {
    let mut create_block = |goto: CTail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
        sym
    };

    match bnd {
        AExpr::Apply { fun, args, typ } => CTail::Seq {
            sym,
            bnd: CExpr::Apply { fun, args, typ },
            tail: Box::new(tail),
        },
        AExpr::FunRef { sym: sym_fn, typ } => CTail::Seq {
            sym,
            bnd: CExpr::FunRef { sym: sym_fn, typ },
            tail: Box::new(tail),
        },
        AExpr::Atom { atm, typ } => CTail::Seq {
            sym,
            bnd: CExpr::Atom { atm, typ },
            tail: Box::new(tail),
        },
        AExpr::Prim { op, args, typ } => CTail::Seq {
            sym,
            bnd: CExpr::Prim { op, args, typ },
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
            explicate_pred(
                *cnd,
                explicate_assign(sym, *thn, CTail::Goto { lbl: tb }, env),
                explicate_assign(sym, *els, CTail::Goto { lbl: tb }, env),
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
                CTail::Goto {
                    lbl: loop_block_sym,
                },
                &mut env,
            );
            env.blocks.insert(loop_block_sym, loop_block);
            CTail::Goto {
                lbl: loop_block_sym,
            }
        }
        AExpr::Break { bdy, .. } => {
            let (break_sym, break_var) = env.break_target.unwrap();
            let break_goto = CTail::Goto { lbl: break_sym };
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
                AExpr::Atom {
                    atm: Atom::Val { val: TLit::Unit },
                    typ: Type::Unit,
                },
                tail,
                env,
            ),
            env,
        ),
        AExpr::Continue { .. } => CTail::Goto {
            lbl: env.continue_target.unwrap(),
        },
        AExpr::Return { bdy, .. } => {
            let tmp = gen_sym("return");
            let tail = CTail::Return {
                expr: Atom::Var { sym: tmp },
                typ: bdy.typ().clone(),
            };
            explicate_assign(tmp, *bdy, tail, env)
        }
        AExpr::Struct {
            sym: sym_,
            fields,
            typ,
        } => CTail::Seq {
            sym,
            bnd: CExpr::Struct {
                sym: sym_,
                fields,
                typ,
            },
            tail: Box::new(tail),
        },
        AExpr::AccessField { strct, field, typ } => CTail::Seq {
            sym,
            bnd: CExpr::AccessField { strct, field, typ },
            tail: Box::new(tail),
        },
    }
}

fn explicate_pred<'p>(
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
            cnd: CExpr::Prim {
                op: Op::EQ,
                args: vec![
                    Atom::Var { sym },
                    Atom::Val {
                        val: TLit::Bool { val: true },
                    },
                ],
                typ: Type::Bool,
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
        AExpr::Prim { op, args, .. } => match op {
            Op::Not => explicate_pred(
                AExpr::Atom {
                    atm: args[0],
                    typ: Type::Bool,
                },
                els,
                thn,
                env,
            ),
            Op::EQ | Op::NE | Op::GT | Op::GE | Op::LT | Op::LE => CTail::IfStmt {
                cnd: CExpr::Prim {
                    op,
                    args,
                    typ: Type::Bool,
                },
                thn: create_block(thn),
                els: create_block(els),
            },
            Op::LAnd | Op::LOr | Op::Xor => {
                let tmp = gen_sym("tmp");
                explicate_assign(
                    tmp,
                    AExpr::Prim {
                        op,
                        args,
                        typ: Type::Bool,
                    },
                    explicate_pred(
                        AExpr::Atom {
                            atm: Atom::Var { sym: tmp },
                            typ: Type::Bool,
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
        AExpr::Let { sym, bnd, bdy, .. } => {
            explicate_assign(sym, *bnd, explicate_pred(*bdy, thn, els, env), env)
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
                *cnd_sub,
                explicate_pred(
                    *thn_sub,
                    CTail::Goto { lbl: thn },
                    CTail::Goto { lbl: els },
                    env,
                ),
                explicate_pred(
                    *els_sub,
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
                AExpr::Apply {
                    fun,
                    args,
                    typ: Type::Bool,
                },
                explicate_pred(
                    AExpr::Atom {
                        atm: Atom::Var { sym: tmp },
                        typ: Type::Bool,
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
                typ: Type::Bool,
            };
            explicate_assign(tmp, cnd, explicate_pred(cnd_, thn, els, env), env)
        }
        AExpr::Seq { stmt, cnt, .. } => explicate_assign(
            gen_sym("ignore"),
            *stmt,
            explicate_pred(*cnt, thn, els, env),
            env,
        ),
        AExpr::AccessField { strct, field, .. } => {
            let tmp = gen_sym("tmp");
            explicate_assign(
                tmp,
                AExpr::AccessField {
                    strct,
                    field,
                    typ: Type::Bool,
                },
                explicate_pred(
                    AExpr::Atom {
                        atm: Atom::Var { sym: tmp },
                        typ: Type::Bool,
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
            atm: Atom::Val {
                val: TLit::U64 { .. } | TLit::I64 {.. } | TLit::Unit,
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
