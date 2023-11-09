use crate::passes::atomize::Atom;
use crate::passes::eliminate::eliminate_params::{eliminate_params, flatten_type};
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::explicate::{CExpr, CTail, PrgExplicated};
use crate::passes::parse::types::Type;
use crate::passes::parse::TypeDef;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use functor_derive::Functor;
use std::collections::HashMap;

// (Old variable name, field name) -> New variable name
pub type Ctx<'p> = HashMap<(UniqueSym<'p>, &'p str), UniqueSym<'p>>;

impl<'p> PrgExplicated<'p> {
    pub fn eliminate(self) -> PrgEliminated<'p> {
        let mut ctx = Ctx::new();

        let fn_params = eliminate_params(self.fn_params, &mut ctx, &self.defs);

        PrgEliminated {
            blocks: self
                .blocks
                .into_iter()
                .map(|(sym, tail)| (sym, eliminate_tail(tail, &mut ctx, &self.defs)))
                .collect(),
            fn_params,
            defs: self.defs,
            entry: self.entry,
        }
    }
}

fn eliminate_tail<'p>(
    tail: CTail<'p>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    match tail {
        CTail::Return { expr, typ } => match expr {
            Atom::Val { val } => ETail::Return {
                exprs: vec![(Atom::Val { val }, typ)],
            },
            Atom::Var { sym } => {
                // dbg!(&sym, &typ);
                // dbg!(flatten_type(sym, &typ, ctx, defs));

                ETail::Return {
                    // exprs: vec![(Atom::Var { sym }, typ)]
                    exprs: flatten_type(sym, &typ, ctx, defs)
                        .fmap(|(sym, typ)| (Atom::Var { sym }, typ)),
                }
            }
        },
        CTail::Seq { sym, bnd, tail } => {
            let tail = eliminate_tail(*tail, ctx, defs);
            eliminate_seq(sym, ctx, bnd, tail, defs)
        }
        CTail::IfStmt { cnd, thn, els } => ETail::IfStmt {
            cnd: map_expr(cnd),
            thn,
            els,
        },
        CTail::Goto { lbl } => ETail::Goto { lbl },
    }
}

fn eliminate_seq<'p>(
    sym: UniqueSym<'p>,
    ctx: &mut Ctx<'p>,
    bnd: CExpr<'p>,
    tail: ETail<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    let bnd = match bnd {
        CExpr::AccessField {
            strct,
            field,
            typ: field_typ,
        } => {
            let strct = strct.var();
            let new_sym = *ctx
                .entry((strct, field))
                .or_insert_with(|| gen_sym(sym.sym));

            return eliminate_seq(
                sym,
                ctx,
                CExpr::Atom {
                    atm: Atom::Var { sym: new_sym },
                    typ: field_typ,
                },
                tail,
                defs,
            );
        }
        CExpr::Apply { fun, args, typ } => {
            let args = args
                .into_iter()
                .flat_map(|(atom, typ)| match atom {
                    Atom::Val { val } => vec![(Atom::Val { val }, typ)],
                    Atom::Var { sym } => flatten_type(sym, &typ, ctx, defs)
                        .into_iter()
                        .map(|(sym, typ)| (Atom::Var { sym }, typ))
                        .collect(),
                })
                .collect();

            CExpr::Apply { fun, args, typ }
        }
        _ => bnd,
    };

    match bnd.typ() {
        Type::Int | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => ETail::Seq {
            syms: vec![sym],
            bnd: map_expr(bnd),
            tail: Box::new(tail),
        },
        Type::Var { sym: def_sym } => match &defs[&def_sym] {
            TypeDef::Struct { fields: def_fields } => match bnd {
                CExpr::Atom { atm, .. } => {
                    def_fields.iter().fold(tail, |tail, (field, field_type)| {
                        let sym_lhs = *ctx.entry((sym, field)).or_insert_with(|| gen_sym(sym.sym));
                        let sym_rhs = *ctx
                            .entry((atm.var(), field))
                            .or_insert_with(|| gen_sym(atm.var().sym));

                        eliminate_seq(
                            sym_lhs,
                            ctx,
                            CExpr::Atom {
                                atm: Atom::Var { sym: sym_rhs },
                                typ: field_type.clone(),
                            },
                            tail,
                            defs,
                        )
                    })
                }
                CExpr::Struct { fields, .. } => {
                    let field_values = fields.into_iter().collect::<HashMap<_, _>>();

                    def_fields.iter().fold(tail, |tail, (field, field_type)| {
                        let sym_lhs = *ctx.entry((sym, field)).or_insert_with(|| gen_sym(sym.sym));
                        let sym_rhs = field_values[field];

                        eliminate_seq(
                            sym_lhs,
                            ctx,
                            CExpr::Atom {
                                atm: sym_rhs,
                                typ: field_type.clone(),
                            },
                            tail,
                            defs,
                        )
                    })
                }
                CExpr::Apply { fun, args, typ } => {
                    let (syms, typs): (Vec<_>, Vec<_>) =
                        flatten_type(sym, &typ, ctx, defs).into_iter().unzip();

                    ETail::Seq {
                        syms,
                        bnd: EExpr::Apply { fun, args, typs },
                        tail: Box::new(tail),
                    }
                }
                CExpr::Prim { .. } | CExpr::FunRef { .. } | CExpr::AccessField { .. } => {
                    unreachable!()
                }
            },
            TypeDef::Enum { .. } => todo!(),
        },
    }
}

fn map_expr(e: CExpr) -> EExpr {
    match e {
        CExpr::Atom { atm, typ } => EExpr::Atom { atm, typ },
        CExpr::Prim { op, args, typ } => EExpr::Prim { op, args, typ },
        CExpr::Apply { fun, args, typ } => EExpr::Apply {
            fun,
            args,
            typs: vec![typ],
        },
        CExpr::FunRef { sym, typ } => EExpr::FunRef { sym, typ },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
