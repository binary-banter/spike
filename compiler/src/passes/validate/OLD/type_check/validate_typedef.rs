use crate::passes::parse::TypeDef;
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::validate_type::validate_type;
use crate::passes::validate::type_check::EnvEntry;
use crate::utils::push_map::PushMap;

pub fn validate_typedef<'p>(
    _sym: &'p str,
    def: TypeDef<&'p str>,
    scope: &PushMap<&str, EnvEntry<'p>>,
) -> Result<TypeDef<&'p str>, TypeError> {
    Ok(match def {
        TypeDef::Struct { fields } => {
            fields
                .iter()
                .try_for_each(|(_, typ)| validate_type(typ, scope))?;
            TypeDef::Struct { fields }
        }
        TypeDef::Enum { .. } => todo!(),
    })
}
