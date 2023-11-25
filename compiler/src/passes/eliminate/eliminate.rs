use crate::passes::atomize::Atom;
use crate::passes::eliminate::eliminate_params::{eliminate_params, flatten_type};
use crate::passes::eliminate::{EExpr, ETail, PrgEliminated};
use crate::passes::explicate::{CExpr, CTail, PrgExplicated};
use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, TypeDef};
use crate::utils::gen_sym::UniqueSym;
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
            std: self.std,
        }
    }
}

fn eliminate_tail<'p>(
    tail: CTail<'p>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    match tail {
        CTail::Return { expr } => match expr.inner {
            Atom::Val { val } => ETail::Return {
                exprs: vec![(Atom::Val { val })],
            },
            Atom::Var { sym } => ETail::Return {
                    exprs: flatten_type(sym, &expr.meta, ctx, defs)
                        .fmap(|(sym, _)| (Atom::Var { sym })),
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
    bnd: Meta<Type<UniqueSym<'p>>, CExpr<'p>>,
    tail: ETail<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> ETail<'p> {
    let typ = bnd.meta;

    // Changes based on RHS
    let bnd = match bnd.inner {
        CExpr::AccessField {
            strct,
            field,
        } => {
            let strct = strct.var();
            let new_sym = *ctx.entry((strct, field)).or_insert_with(|| sym.fresh());

            return eliminate_seq(
                sym,
                ctx,
                Meta{ meta: typ, inner: CExpr::Atom {
                    atm: Atom::Var { sym: new_sym },
                }},
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
                bnd: Meta { meta: vec![typ], inner: map_expr(bnd) },
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
                            Meta { meta: field_type.clone(), inner: CExpr::Atom {
                                atm: Atom::Var { sym: sym_rhs },
                            }},
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
                            Meta { meta: field_type.clone(), inner: CExpr::Atom {
                                atm: atom_rhs,
                            }},
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
                        bnd: Meta { meta: typs, inner: EExpr::Apply { fun, args: args.into_iter().map(|(atm, _)| atm).collect() } },
                        tail: Box::new(tail),
                    }
                }
                CExpr::BinaryOp {..} | CExpr::UnaryOp {..} | CExpr::FunRef { .. } | CExpr::AccessField { .. } => {
                    unreachable!()
                }
            },
            TypeDef::Enum { .. } => todo!(),
        },
    }
}

fn map_expr(e: CExpr) -> EExpr {
    match e {
        CExpr::Atom { atm } => EExpr::Atom { atm },
        CExpr::BinaryOp { op, exprs } => EExpr::BinaryOp {
            op,
            exprs,
        },
        CExpr::UnaryOp { op, expr } => EExpr::UnaryOp {
            op,
            expr,
        },
        CExpr::Apply { fun, args } => EExpr::Apply {
            fun,
            args: args.fmap(|(arg, _)| arg),
        },
        CExpr::FunRef { sym } => EExpr::FunRef { sym },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
