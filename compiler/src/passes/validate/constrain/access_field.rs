use crate::passes::parse::{Meta, Span, TypeDef};
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::expr::constrain_expr;

pub fn constrain_access_field<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    strct: Meta<Span, ExprUniquified<'p>>,
    field: Meta<Span, &'p str>,
) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let strct = constrain_expr(strct, env)?;

    let PartialType::Var { sym } = env.uf.get(strct.meta.index) else {
        return Err(TypeError::SymbolShouldBeStruct {
            span: strct.meta.span,
        });
    };

    let EnvEntry::Def {
        def: TypeDef::Struct {
            fields: def_fields, ..
        },
    } = &env.scope[sym]
    else {
        return Err(TypeError::SymbolShouldBeStruct {
            span: strct.meta.span,
        });
    };

    let Some((_, typ)) = def_fields.iter().find(|(sym, _)| sym.inner == field.inner) else {
        return Err(TypeError::UnknownStructField {
            sym: field.inner.to_string(),
            span: field.meta,
        });
    };

    let index = env.uf.type_to_index(typ.clone());
    Ok(Meta {
        meta: CMeta { span, index },
        inner: ExprConstrained::AccessField {
            strct: Box::new(strct),
            field,
        },
    })
}
