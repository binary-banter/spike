use crate::passes::atomize::{AExpr, Atom, DefAtomized, PrgAtomized};
use crate::passes::explicate::{CExpr, CTail, PrgExplicated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{BinaryOp, Def, Meta, TypeDef, UnaryOp};
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
            std: self.std
        }
    }
}

fn explicate_def<'p>(
    def: DefAtomized<'p>,
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

fn explicate_tail<'p>(expr: Meta<Type<UniqueSym<'p>>, AExpr<'p>>, env: &mut Env<'_, 'p>) -> CTail<'p> {
    let tmp = gen_sym("return");
    let tail = CTail::Return {
        expr: Atom::Var { sym: tmp },
    };
    explicate_assign(tmp, expr, tail, env)
}

fn explicate_assign<'p>(
    sym: UniqueSym<'p>,
    bnd: Meta<Type<UniqueSym<'p>>, AExpr<'p>>,
    tail: CTail<'p>,
    env: &mut Env<'_, 'p>,
) -> CTail<'p> {
    let mut create_block = |goto: CTail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
        sym
    };

    match bnd.inner {
        AExpr::Apply { fun, args } => CTail::Seq {
            sym,
            bnd: Meta { meta: bnd.meta, inner: CExpr::Apply { fun, args } },
            tail: Box::new(tail),
        },
        AExpr::FunRef { sym: sym_fn } => CTail::Seq {
            sym,
            bnd: Meta { meta: bnd.meta, inner: CExpr::FunRef { sym: sym_fn }},
            tail: Box::new(tail),
        },
        AExpr::Atom { atm } => CTail::Seq {
            sym,
            bnd: Meta { meta: bnd.meta, inner: CExpr::Atom { atm }},
            tail: Box::new(tail),
        },
        AExpr::BinaryOp { op, exprs} => CTail::Seq {
            sym,
            bnd: Meta{ meta: bnd.meta, inner: CExpr::BinaryOp { op, exprs }},
            tail: Box::new(tail),
        },
        AExpr::UnaryOp { op, expr } => CTail::Seq {
            sym,
            bnd: Meta{ meta: bnd.meta, inner: CExpr::UnaryOp { op, expr }},
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
                Meta{ meta: bnd.meta, inner: AExpr::Atom {
                    atm: Atom::Val { val: TLit::Unit },
                }},
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
            };
            explicate_assign(tmp, *bdy, tail, env)
        }
        AExpr::Struct {
            sym: sym_,
            fields,
        } => CTail::Seq {
            sym,
            bnd: Meta{ meta: bnd.meta, inner: CExpr::Struct {
                sym: sym_,
                fields,
            }},
            tail: Box::new(tail),
        },
        AExpr::AccessField { strct, field } => CTail::Seq {
            sym,
            bnd: Meta{ meta: bnd.meta, inner: CExpr::AccessField { strct, field } },
            tail: Box::new(tail),
        },
    }
}

fn explicate_pred<'p>(
    cnd: Meta<Type<UniqueSym<'p>>, AExpr<'p>>,
    thn: CTail<'p>,
    els: CTail<'p>,
    env: &mut Env<'_, 'p>,
) -> CTail<'p> {
    let mut create_block = |goto: CTail<'p>| {
        let sym = gen_sym("tmp");
        env.blocks.insert(sym, goto);
        sym
    };
    
    match cnd.inner {
        AExpr::Atom {
            atm: Atom::Var { sym },
            ..
        } => CTail::IfStmt {
            cnd: Meta{ meta: Type::Bool, inner: CExpr::BinaryOp {
                op: BinaryOp::EQ,
                exprs: [Atom::Var { sym }, Atom::Val { val: TLit::Bool { val: true } }]
            }},
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
        AExpr::BinaryOp { op, exprs } => {
            match op {
                BinaryOp::LAnd | BinaryOp::LOr | BinaryOp::Xor => {
                    let tmp = gen_sym("tmp");
                    explicate_assign(
                        tmp,
                        Meta{ meta: Type::Bool, inner: AExpr::BinaryOp {
                            op,
                            exprs,
                        }},
                        explicate_pred(
                            Meta{ meta: Type::Bool, inner: AExpr::Atom {
                                atm: Atom::Var { sym: tmp },
                            }},
                            thn,
                            els,
                            env,
                        ),
                        env,
                    )
                },
                BinaryOp::GT | BinaryOp::GE | BinaryOp::EQ |BinaryOp::LE | BinaryOp::LT | BinaryOp::NE => CTail::IfStmt {
                            cnd: Meta{ meta: Type::Bool, inner: CExpr::BinaryOp {
                                op,
                                exprs,
                            }},
                            thn: create_block(thn),
                            els: create_block(els),
                        },
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod=> unreachable!("Unexpected `BinaryOp` in predicate position."),
            }
        },
        AExpr::UnaryOp { op, expr } => {
            match op {
                UnaryOp::Not => explicate_pred(
                    Meta{ meta: Type::Bool, inner: AExpr::Atom {
                        atm: expr,
                    }},
                    els,
                    thn,
                    env,
                ),
                UnaryOp::Neg => unreachable!("Unexpected `UnaryOp` in predicate position."),
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
                Meta{ meta: Type::Bool, inner: AExpr::Apply {
                    fun,
                    args,
                }},
                explicate_pred(
                    Meta{ meta: Type::Bool, inner: AExpr::Atom {
                        atm: Atom::Var { sym: tmp },
                    }},
                    thn,
                    els,
                    env,
                ),
                env,
            )
        }
        AExpr::Loop { .. } => {
            let tmp = gen_sym("tmp");
            let cnd_ = Meta{ meta: Type::Bool, inner: AExpr::Atom {
                atm: Atom::Var { sym: tmp },
            }};
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
                Meta{ meta: Type::Bool, inner: AExpr::AccessField {
                    strct,
                    field,
                }},
                explicate_pred(
                    Meta{ meta: Type::Bool, inner: AExpr::Atom {
                        atm: Atom::Var { sym: tmp },
                    }},
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
