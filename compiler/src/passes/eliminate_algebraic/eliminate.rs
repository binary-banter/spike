use crate::passes::atomize::Atom;
use crate::passes::eliminate_algebraic::eliminate_params::{eliminate_params, flatten_params};
use crate::passes::eliminate_algebraic::{EExpr, PrgEliminated};
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
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
    tail: Tail<'p, CExpr<'p>>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
) -> Tail<'p, EExpr<'p>> {
    match tail {
        // TODO how to pass structs out of functions?
        Tail::Return { expr } => Tail::Return { expr },
        Tail::Seq { sym, bnd, tail } => {
            let tail = eliminate_tail(*tail, ctx, defs);
            eliminate_seq(sym, ctx, bnd, tail, defs)
        }
        Tail::IfStmt { cnd, thn, els } => Tail::IfStmt {
            cnd: map_expr(cnd),
            thn,
            els,
        },
        Tail::Goto { lbl } => Tail::Goto { lbl },
    }
}

fn eliminate_seq<'p>(
    sym: UniqueSym<'p>,
    ctx: &mut Ctx<'p>,
    bnd: CExpr<'p>,
    tail: Tail<'p, EExpr<'p>>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
) -> Tail<'p, EExpr<'p>> {
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
            fn_typ,
            typ,
        } => {
            #[rustfmt::skip]
            let Type::Fn { params, typ: rtrn_typ} = fn_typ else {
                unreachable!("fn_type should be a function type")
            };

            let (args, params): (Vec<_>, Vec<_>) = args
                .into_iter()
                .zip(params.into_iter())
                .flat_map(|(atom, typ)| {
                    match atom {
                        Atom::Val { val } => vec![(Atom::Val { val }, typ)],
                        Atom::Var { sym } => {
                            flatten_params(sym, &typ, ctx, defs)
                                .into_iter()
                                .map(|(sym, typ)| (Atom::Var { sym }, typ))
                                .collect()
                        }
                    }
                })
                .unzip();

            CExpr::Apply {
                fun,
                args,
                fn_typ: Type::Fn {
                    params,
                    typ: rtrn_typ.clone(),
                },
                typ,
            }
        }
        _ => bnd,
    };

    match bnd.typ() {
        Type::Int | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => Tail::Seq {
            sym,
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
                    CExpr::Apply { .. } => todo!(),
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
            fn_typ,
        } => EExpr::Apply {
            fun,
            args,
            typ,
            fn_typ,
        },
        CExpr::FunRef { sym, typ } => EExpr::FunRef { sym, typ },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
