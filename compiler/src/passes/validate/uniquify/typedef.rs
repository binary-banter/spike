use crate::passes::parse::{Def, Meta, Span, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::r#type::uniquify_type;
use crate::passes::validate::uniquify::try_get;
use crate::passes::validate::DefUniquified;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

pub fn uniquify_typedef<'p>(
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
    sym: Meta<Span, &'p str>,
    def: TypeDef<Meta<Span, &'p str>, Meta<Span, &'p str>>,
) -> Result<DefUniquified<'p>, TypeError> {
    let def = match def {
        TypeDef::Struct { fields } => TypeDef::Struct {
            fields: fields
                .into_iter()
                .map(|(sym, typ)| Ok((sym, uniquify_type(typ, scope)?)))
                .collect::<Result<_, _>>()?,
        },
        TypeDef::Enum { .. } => todo!(),
    };

    Ok(Def::TypeDef {
        sym: try_get(sym, scope)?,
        def,
    })
}
