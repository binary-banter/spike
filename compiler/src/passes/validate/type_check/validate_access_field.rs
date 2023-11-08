use crate::passes::parse::types::Type;
use crate::passes::parse::{Expr, Spanned, TypeDef};
use crate::passes::validate::type_check::error::TypeError;
use crate::passes::validate::type_check::error::TypeError::*;
use crate::passes::validate::type_check::validate_expr::validate_expr;
use crate::passes::validate::type_check::{Env, EnvEntry};
use crate::passes::validate::TExpr;

pub fn validate_access_field<'p>(
    strct: Spanned<Expr<'p>>,
    field: &'p str,
    span: (usize, usize),
    env: &mut Env<'_, 'p>,
) -> Result<Spanned<TExpr<'p, &'p str>>, TypeError> {
    let strct = validate_expr(strct, env)?;

    let Type::Var { sym } = strct.inner.typ() else {
        return Err(TypeShouldBeStruct {
            typ: strct.inner.typ().clone().fmap(str::to_string),
        });
    };

    let EnvEntry::Def {
        def: TypeDef::Struct {
            fields: def_fields, ..
        },
    } = &env.scope[sym]
    else {
        return Err(VariableShouldBeStruct {
            sym: sym.to_string(),
        });
    };

    let Some((_, typ)) = def_fields.iter().find(|&(sym, _)| *sym == field) else {
        return Err(UnknownStructField {
            sym: sym.to_string(),
        });
    };

    Ok(Spanned {
        span,
        inner: TExpr::AccessField {
            strct: Box::new(strct.inner),
            field,
            typ: typ.clone(),
        },
    })
}
