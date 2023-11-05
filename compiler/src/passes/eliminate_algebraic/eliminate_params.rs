use crate::passes::eliminate_algebraic::eliminate::Ctx;
use crate::passes::parse::types::Type;
use crate::passes::parse::{Param, TypeDef};
use crate::utils::gen_sym::{gen_sym, UniqueSym};
use std::collections::HashMap;

pub fn eliminate_params<'p>(
    fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
) -> HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>> {
    fn_params
        .into_iter()
        .map(|(sym, params)| {
            (
                sym,
                params
                    .into_iter()
                    .flat_map(|param| {
                        flatten_params(param.sym, &param.typ, ctx, defs)
                            .into_iter()
                            .map(move |(sym, typ)| Param {
                                sym,
                                typ,
                                mutable: param.mutable,
                            })
                    })
                    .collect(),
            )
        })
        .collect()
}

/// Given an expression of `param_sym: param_type`
/// Returns a flattened Vec of expressions of `(UniqueSym<'p>, Type<UniqueSym<'p>>)`
pub fn flatten_params<'p>(
    param_sym: UniqueSym<'p>,
    param_type: &Type<UniqueSym<'p>>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<'p, UniqueSym<'p>>>,
) -> Vec<(UniqueSym<'p>, Type<UniqueSym<'p>>)> {
    match param_type {
        Type::Int | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => {
            vec![(param_sym, param_type.clone())]
        }
        Type::Var { sym } => match &defs[&sym] {
            TypeDef::Struct { fields } => fields
                .iter()
                .flat_map(|(field_name, field_type)| {
                    let new_sym = *ctx
                        .entry((param_sym, field_name))
                        .or_insert_with(|| gen_sym(param_sym.sym));

                    flatten_params(new_sym, field_type, ctx, defs).into_iter()
                })
                .collect(),
            TypeDef::Enum { .. } => todo!(),
        },
    }
}
