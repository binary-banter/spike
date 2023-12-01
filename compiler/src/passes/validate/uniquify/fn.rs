use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, ExprParsed, Param, Spanned};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::expr::uniquify_expr;
use crate::passes::validate::uniquify::gen_spanned_sym;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::{uniquify, DefUniquified};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;
use std::collections::HashMap;

pub fn uniquify_fn<'p>(
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
    sym: Spanned<&'p str>,
    params: Vec<Param<Spanned<&'p str>>>,
    typ: Type<Spanned<&'p str>>,
    bdy: Spanned<ExprParsed<'p>>,
) -> Result<DefUniquified<'p>, TypeError> {
    // Generate unique names for the parameters.
    let iterator = params
        .iter()
        .map(|param| (param.sym.inner, gen_spanned_sym(param.sym.clone()).inner));

    // Push the parameters into scope and uniquify the function.
    scope.push_iter(iterator, |scope| {
        // Collect uniquified parameters.
        let params = params
            .iter()
            .map(|param| uniquify_param(param, scope))
            .collect::<Result<Vec<_>, _>>()?;

        // Uniquify body of the function.
        let bdy = uniquify_expr(bdy, scope)?;

        // Check that there are no duplicate parameter names.
        let mut param_syms = HashMap::new();
        for param in &params {
            // Span of the previously defined duplicate.
            if let Some(prev_span) = param_syms.insert(param.sym.inner, param.sym.meta) {
                return Err(TypeError::DuplicateArg {
                    span1: prev_span,
                    span2: param.sym.meta,
                    sym: param.sym.inner.sym.to_string(),
                });
            }
        }

        Ok(Def::Fn {
            sym: uniquify::try_get(sym, scope)?,
            params,
            typ: uniquify_type(typ, scope)?,
            bdy,
        })
    })
}

fn uniquify_param<'p>(
    param: &Param<Spanned<&'p str>>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<Param<Spanned<UniqueSym<'p>>>, TypeError> {
    Ok(Param {
        sym: uniquify::try_get(param.sym.clone(), scope)?,
        mutable: param.mutable,
        typ: uniquify_type(param.typ.clone(), scope)?,
    })
}
