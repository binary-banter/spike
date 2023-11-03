use crate::passes::eliminate_algebraic::{EExpr, PrgEliminated};
use crate::passes::explicate::{CExpr, PrgExplicated, Tail};
use crate::passes::parse::types::Type;
use crate::passes::parse::TypeDef;
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;
use crate::passes::atomize::Atom;

// (Old variable name, field name) -> New variable name
type Ctx<'p> = HashMap<(UniqueSym<'p>, &'p str), UniqueSym<'p>>;

impl<'p> PrgExplicated<'p> {
    pub fn eliminate(self) -> PrgEliminated<'p> {
        let mut ctx = Ctx::new();

        PrgEliminated {
            blocks: self
                .blocks
                .into_iter()
                .map(|(sym, tail)| (sym, eliminate_tail(tail, &mut ctx, &self.defs)))
                .collect(),
            fn_params: self.fn_params,
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
            eliminate_seq(
                sym,
                ctx,
                bnd,
                tail,
                defs,
            )
        },
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
    if let CExpr::AccessField { strct, field, typ: field_typ } = bnd {
        let strct = strct.var();
        let new_sym = *ctx.entry((strct, field)).or_insert_with(|| gen_sym(sym.sym));

        return eliminate_seq(sym, ctx,CExpr::Atom { atm: Atom::Var { sym: new_sym }, typ: field_typ }, tail, defs );
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
                        fields.into_iter().map(|&(field, _)| {
                            let new_sym = *ctx.entry((v, field)).or_insert_with(|| gen_sym(sym.sym));
                            (field, Atom::Var { sym: new_sym })
                        }).collect()
                    }
                    CExpr::Prim { .. } => unreachable!(),
                    CExpr::Apply { .. } => todo!(),
                    CExpr::FunRef { .. } => unreachable!(),
                    CExpr::Struct { fields, .. } => fields.into_iter().collect(),
                    CExpr::AccessField { .. } => todo!(),
                };

                fields.iter().fold(tail, |tail, (field, field_type)| {
                    let new_sym = *ctx.entry((sym, field)).or_insert_with(|| gen_sym(sym.sym));
                    eliminate_seq(new_sym, ctx, CExpr::Atom { atm: field_values[field], typ: field_type.clone() }, tail, defs)
                })
            },
            TypeDef::Enum { .. } => todo!()
        },
    }
}

fn map_expr(e: CExpr) -> EExpr {
    match e {
        CExpr::Atom { atm, typ } => EExpr::Atom { atm, typ },
        CExpr::Prim { op, args, typ } => EExpr::Prim { op, args, typ },
        CExpr::Apply { fun, args, typ } => EExpr::Apply { fun, args, typ },
        CExpr::FunRef { sym, typ } => EExpr::FunRef { sym, typ },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
