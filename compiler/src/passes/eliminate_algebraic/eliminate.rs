use crate::passes::atomize::Atom;
use crate::passes::eliminate_algebraic::eliminate_params::{eliminate_params, flatten_params};
use crate::passes::eliminate_algebraic::{EExpr, ETail, PrgEliminated};
use crate::passes::explicate::{CExpr, PrgExplicated, CTail};
use crate::passes::parse::types::Type;
use crate::passes::parse::TypeDef;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
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
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
) -> ETail<'p> {
    match tail {
        CTail::Return { expr } => ETail::Return { expr: vec![expr] },
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
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
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
        CExpr::Apply {
            fun,
            args,
            typ,
        } => {
            let args = args
                .into_iter()
                .flat_map(|(atom, typ)| match atom {
                    Atom::Val { val } => vec![(Atom::Val { val }, typ)],
                    Atom::Var { sym } => flatten_params(sym, &typ, ctx, defs)
                        .into_iter()
                        .map(|(sym, typ)| (Atom::Var { sym }, typ))
                        .collect(),
                }).collect();

            CExpr::Apply {
                fun,
                args,
                typ,
            }
        }
        _ => bnd,
    };

    match bnd.typ() {
        Type::Int | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => ETail::Seq {
            sym: vec![sym],
            bnd: map_expr(bnd),
            tail: Box::new(tail),
        },
        Type::Var { sym: def_sym } => match &defs[&def_sym] {
            TypeDef::Struct { fields } => {
                let field_values: HashMap<_, _> = match bnd {
                    CExpr::Atom { atm, .. } => {
                        let v = atm.var();
                        fields
                            .iter()
                            .map(|&(field, _)| {
                                let new_sym =
                                    *ctx.entry((v, field)).or_insert_with(|| gen_sym(v.sym));
                                (field, Atom::Var { sym: new_sym })
                            })
                            .collect()
                    }
                    CExpr::Struct { fields, .. } => fields.into_iter().collect(),
                    CExpr::Apply { fun, args, typ } => {


                        // sym.field1 =
                        // sym.field2 =

                        todo!()
                    },
                    CExpr::Prim { .. } | CExpr::FunRef { .. } | CExpr::AccessField { .. } => {
                        unreachable!()
                    }
                };

                fields.iter().fold(tail, |tail, (field, field_type)| {
                    let new_sym = *ctx.entry((sym, field)).or_insert_with(|| gen_sym(sym.sym));
                    eliminate_seq(
                        new_sym,
                        ctx,
                        CExpr::Atom {
                            atm: field_values[field],
                            typ: field_type.clone(),
                        },
                        tail,
                        defs,
                    )
                })
            }
            TypeDef::Enum { .. } => todo!(),
        },
    }
}

fn map_expr(e: CExpr) -> EExpr {
    match e {
        CExpr::Atom { atm, typ } => EExpr::Atom { atm, typ },
        CExpr::Prim { op, args, typ } => EExpr::Prim { op, args, typ },
        CExpr::Apply {
            fun,
            args,
            typ,
        } => EExpr::Apply {
            fun,
            args,
            typ: vec![typ], //TODO baaaaad implementation, baaaaaaad
        },
        CExpr::FunRef { sym, typ } => EExpr::FunRef { sym, typ },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
