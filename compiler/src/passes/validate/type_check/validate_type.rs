use crate::passes::parse::types::Type;
use crate::passes::validate::type_check::EnvEntry;
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::utils::expect::expect;
use crate::utils::push_map::PushMap;

/// Verifies that the given type exists in the current scope.
pub fn validate_type<'p>(
    typ: &'p Type<&'p str>,
    scope: &PushMap<&str, EnvEntry<'p>>,
) -> Result<(), TypeError> {
    match typ {
        Type::Int | Type::Bool | Type::Unit | Type::Never => {}
        Type::Fn { typ, params } => {
            validate_type(typ, scope)?;

            for typ in params {
                validate_type(typ, scope)?;
            }
        }
        Type::Var { sym } => {
            expect(
                scope.contains(sym),
                UndeclaredVar {
                    sym: sym.to_string(),
                },
            )?;
        }
    }

    Ok(())
}
