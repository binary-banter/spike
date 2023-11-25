use crate::passes::parse::{Def, DefParsed};
use crate::passes::validate::DefUniquified;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uniquify::r#fn::uniquify_fn;
use crate::passes::validate::uniquify::typedef::uniquify_typedef;
use crate::utils::gen_sym::UniqueSym;
use crate::utils::push_map::PushMap;

pub fn uniquify_def<'p>(
    def: DefParsed<'p>,
    scope: &mut PushMap<&'p str, UniqueSym<'p>>,
) -> Result<DefUniquified<'p>, TypeError> {
    match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => uniquify_fn(scope, sym, params, typ, bdy),
        Def::TypeDef { sym, def } => uniquify_typedef(scope, sym, def),
    }
}
