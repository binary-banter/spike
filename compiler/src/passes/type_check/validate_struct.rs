use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Expr};
use crate::passes::type_check::check::{Env, EnvEntry};
use crate::passes::type_check::error::TypeError;
use crate::passes::type_check::error::TypeError::*;
use crate::passes::type_check::util;
use crate::utils::expect::expect;
use std::collections::{HashMap, HashSet};

pub fn validate_struct<'p>(
    env: &mut Env<'_, 'p>,
    sym: &'p str,
    provided_fields: &Vec<(&str, Expr<'p, &'p str>)>,
) -> Result<Type<&'p str>, TypeError> {
    let entry = env.scope.get(&sym).ok_or(UndeclaredVar {
        sym: sym.to_string(),
    })?;

    let EnvEntry::Def {
        def: Def::Struct {
            fields: def_fields, ..
        },
    } = entry
    else {
        return Err(VariableShouldBeStruct {
            sym: sym.to_string(),
        });
    };

    let mut new_provided_fields = HashSet::new();
    let def_fields = def_fields
        .iter()
        .map(|(k, v)| (*k, v))
        .collect::<HashMap<_, _>>();

    for (field, expr) in provided_fields {
        expect(
            new_provided_fields.insert(field),
            VariableConstructDuplicateField {
                sym: field.to_string(),
            },
        )?;

        if let Some(typ) = def_fields.get(field) {
            util::expect_type(expr, (*typ).clone(), env)?;
        } else {
            return Err(UnknownStructField {
                sym: field.to_string(),
            });
        }
    }

    for field in def_fields.keys() {
        expect(
            new_provided_fields.contains(field),
            VariableConstructMissingField {
                sym: field.to_string(),
            },
        )?;
    }

    Ok(Type::Var { sym })
}
