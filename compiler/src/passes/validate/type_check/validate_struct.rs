use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned, TypeDef};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::expect_type;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{Env, EnvEntry};
use crate::passes::validate::TExpr;
use crate::utils::expect::expect;
use std::collections::{HashMap, HashSet};

pub fn validate_struct<'p>(
    env: &mut Env<'_, 'p>,
    sym: &'p str,
    fields: Vec<(&'p str, Spanned<Expr<'p>>)>,
) -> Result<TExpr<'p, &'p str>, TypeError> {
    let entry = env.scope.get(&sym).ok_or(UndeclaredVar {
        sym: sym.to_string(),
    })?;

    #[rustfmt::skip]
    let EnvEntry::Def { def: TypeDef::Struct { fields: def_fields, .. } } = &entry else {
        return Err(VariableShouldBeStruct { sym: sym.to_string() });
    };

    let mut new_provided_fields = HashSet::new();
    let def_fields = def_fields
        .iter()
        .map(|(k, v)| (*k, v.clone()))
        .collect::<HashMap<_, _>>();

    let fields = fields
        .into_iter()
        .map(|(field, expr)| {
            let expr = validate_expr(expr.expr, env)?;

            expect(
                new_provided_fields.insert(field),
                VariableConstructDuplicateField {
                    sym: field.to_string(),
                },
            )?;

            if let Some(typ) = def_fields.get(field) {
                expect_type(&expr, typ)?;
            } else {
                return Err(UnknownStructField {
                    sym: field.to_string(),
                });
            }

            Ok((field, expr))
        })
        .collect::<Result<Vec<_>, _>>()?;

    for field in def_fields.keys() {
        expect(
            new_provided_fields.contains(field),
            VariableConstructMissingField {
                sym: field.to_string(),
            },
        )?;
    }

    Ok(TExpr::Struct {
        sym,
        fields,
        typ: Type::Var { sym },
    })
}
