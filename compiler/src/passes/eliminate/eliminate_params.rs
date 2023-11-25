use crate::passes::eliminate::eliminate::Ctx;
use crate::passes::parse::types::Type;
use crate::passes::parse::{Param, TypeDef};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

pub fn eliminate_params<'p>(
    fn_params: HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> HashMap<UniqueSym<'p>, Vec<Param<UniqueSym<'p>>>> {
    fn_params
        .into_iter()
        .map(|(sym, params)| {
            (
                sym,
                params
                    .into_iter()
                    .flat_map(|param| {
                        flatten_type(param.sym, &param.typ, ctx, defs)
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

/// Given an expression of `sym: typ`
/// Returns a flattened Vec of expressions of `(UniqueSym<'p>, Type<UniqueSym<'p>>)`
pub fn flatten_type<'p>(
    sym: UniqueSym<'p>,
    typ: &Type<UniqueSym<'p>>,
    ctx: &mut Ctx<'p>,
    defs: &HashMap<UniqueSym<'p>, TypeDef<UniqueSym<'p>, &'p str>>,
) -> Vec<(UniqueSym<'p>, Type<UniqueSym<'p>>)> {
    match typ {
        Type::I64 | Type::U64 | Type::Bool | Type::Unit | Type::Never | Type::Fn { .. } => {
            vec![(sym, typ.clone())]
        }
        Type::Var { sym: def_sym } => match &defs[&def_sym] {
            TypeDef::Struct { fields } => fields
                .iter()
                .flat_map(|(field_name, field_type)| {
                    let new_sym = *ctx.entry((sym, field_name)).or_insert_with(|| sym.fresh());

                    flatten_type(new_sym, field_type, ctx, defs).into_iter()
                })
                .collect(),
            TypeDef::Enum { .. } => todo!(),
        },
    }
}
