use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use itertools::Itertools;
use crate::passes::parse::types::Int;

#[derive(Debug, Clone)]
pub enum PartialType<'p> {
    Int(Int),
    IntAmbiguous,
    Bool,
    Unit,
    Never,
    Var {
        sym: UniqueSym<'p>,
    },
    Fn {
        params: Vec<UnionIndex>,
        typ: UnionIndex,
    },
}

impl<'p> PartialType<'p> {
    pub fn to_string(&self, uf: &mut UnionFind<PartialType>) -> String {
        match self {
            PartialType::Int(int) => int.to_string(),
            PartialType::IntAmbiguous => "{int}".to_string(),
            PartialType::Bool => "Bool".to_string(),
            PartialType::Unit => "Unit".to_string(),
            PartialType::Never => "Never".to_string(),
            PartialType::Var { sym } => sym.sym.to_string(),
            PartialType::Fn { params, typ } => {
                let params_string = params
                    .iter()
                    .map(|index| {
                        let pt = uf.get(*index).clone();
                        pt.to_string(uf)
                    })
                    .format(", ")
                    .to_string();
                let pt = uf.get(*typ).clone();
                let typ_string = pt.to_string(uf);
                format!("fn({params_string}) -> {typ_string}")
            }
        }
    }
}

#[allow(clippy::result_unit_err)]
pub fn combine_partial_types<'p>(
    a: PartialType<'p>,
    b: PartialType<'p>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<PartialType<'p>, ()> {
    let typ = match (a, b) {
        (PartialType::Int(int_lhs), PartialType::Int(int_rhs)) if int_lhs == int_rhs => PartialType::Int(int_lhs),
        (PartialType::Int(int), PartialType::IntAmbiguous) => PartialType::Int(int),
        (PartialType::IntAmbiguous, PartialType::Int(int)) => PartialType::Int(int),
        (PartialType::IntAmbiguous, PartialType::IntAmbiguous) => PartialType::IntAmbiguous,
        (PartialType::Bool, PartialType::Bool) => PartialType::Bool,
        (PartialType::Unit, PartialType::Unit) => PartialType::Unit,
        (PartialType::Never, t) => t.clone(),
        (t, PartialType::Never) => t.clone(),
        (PartialType::Var { sym: sym_a }, PartialType::Var { sym: sym_b }) if sym_a == sym_b => {
            PartialType::Var { sym: sym_a }
        }
        (
            PartialType::Fn {
                params: params_a,
                typ: typ_a,
            },
            PartialType::Fn {
                params: params_b,
                typ: typ_b,
            },
        ) => {
            if params_a.len() != params_b.len() {
                return Err(());
            }

            let params = params_a
                .into_iter()
                .zip(params_b)
                .map(|(param_a, param_b)| uf.try_union_by(param_a, param_b, combine_partial_types))
                .collect::<Result<_, _>>()?;

            let typ = uf.try_union_by(typ_a, typ_b, combine_partial_types)?;

            PartialType::Fn { params, typ }
        }
        _ => return Err(()),
    };

    Ok(typ)
}
