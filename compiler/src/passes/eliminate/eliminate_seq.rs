use crate::passes::atomize::Atom;
use crate::passes::eliminate::eliminate::Ctx;
use crate::passes::eliminate::eliminate_expr::eliminate_expr;
use crate::passes::eliminate::eliminate_params::flatten_type;
use crate::passes::eliminate::{EExpr, ETail};
use crate::passes::explicate::CExpr;
use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, TypeDef, Typed};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub fn eliminate_seq<'p>(
    sym: UniqueSym<'p>,
    ctx: &mut Ctx<'p>,
    bnd: Typed<'p, CExpr<'p>>,
    tail: ETail<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    let typ = bnd.meta;

    // Changes based on RHS
    let bnd = match bnd.inner {
        CExpr::AccessField { strct, field } => {
            let strct = strct.var();
            let new_sym = *ctx.entry((strct, field)).or_insert_with(|| sym.fresh());

            return eliminate_seq(
                sym,
                ctx,
                Meta {
                    meta: typ,
                    inner: CExpr::Atom {
                        atm: Atom::Var { sym: new_sym },
                    },
                },
                tail,
                defs,
            );
        }
        CExpr::Apply { fun, args } => {
            // Flatten the arguments. This is trivial for `Val` atoms, but for `Var` atoms `flatten_type` is used.
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

            CExpr::Apply { fun, args }
        }
        inner => inner,
    };

    // Changes based on LHS
    match typ {
        // No changes needed
        Type::I64 | Type::U64 | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => {
            ETail::Seq {
                syms: vec![sym],
                bnd: Meta {
                    meta: vec![typ],
                    inner: eliminate_expr(bnd),
                },
                tail: Box::new(tail),
            }
        }
        Type::Var { sym: def_sym } => match &defs[&def_sym] {
            // Changes needed, since LHS is a struct
            TypeDef::Struct { fields: def_fields } => match bnd {
                CExpr::Atom { atm, .. } => {
                    def_fields.iter().fold(tail, |tail, (field, field_type)| {
                        let sym_lhs = *ctx.entry((sym, field)).or_insert_with(|| sym.fresh());
                        let sym_rhs = *ctx
                            .entry((atm.var(), field))
                            .or_insert_with(|| atm.var().fresh());

                        eliminate_seq(
                            sym_lhs,
                            ctx,
                            Meta {
                                meta: field_type.clone(),
                                inner: CExpr::Atom {
                                    atm: Atom::Var { sym: sym_rhs },
                                },
                            },
                            tail,
                            defs,
                        )
                    })
                }
                CExpr::Struct { fields, .. } => {
                    let field_values = fields.into_iter().collect::<HashMap<_, _>>();

                    def_fields.iter().fold(tail, |tail, (field, field_type)| {
                        let sym_lhs = *ctx.entry((sym, field)).or_insert_with(|| sym.fresh());
                        let atom_rhs = field_values[field];

                        eliminate_seq(
                            sym_lhs,
                            ctx,
                            Meta {
                                meta: field_type.clone(),
                                inner: CExpr::Atom { atm: atom_rhs },
                            },
                            tail,
                            defs,
                        )
                    })
                }
                CExpr::Apply { fun, args } => {
                    let (syms, typs): (Vec<_>, Vec<_>) =
                        flatten_type(sym, &typ, ctx, defs).into_iter().unzip();

                    ETail::Seq {
                        syms,
                        bnd: Meta {
                            meta: typs,
                            inner: EExpr::Apply {
                                fun,
                                args: args.into_iter().map(|(atm, _)| atm).collect(),
                            },
                        },
                        tail: Box::new(tail),
                    }
                }
                CExpr::BinaryOp { .. }
                | CExpr::UnaryOp { .. }
                | CExpr::FunRef { .. }
                | CExpr::AccessField { .. }
                | CExpr::Asm { .. } => {
                    unreachable!()
                }
            },
            TypeDef::Enum { .. } => todo!(),
        },
    }
}
