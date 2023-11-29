use crate::passes::parse::{Constrained, Span, Spanned, TypeDef};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};
use crate::utils::expect::expect;
use crate::utils::gen_sym::UniqueSym;
use std::collections::{HashMap, HashSet};

pub fn constrain_struct<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    sym: Spanned<UniqueSym<'p>>,
    fields: Vec<(Spanned<&'p str>, Spanned<ExprUniquified<'p>>)>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    // Get the `EnvEntry` from the scope.
    // This should exist after uniquify, but could potentially not be a struct definition.
    let EnvEntry::Def {
        def: TypeDef::Struct { fields: def_fields },
    } = &env.scope[&sym.inner]
    else {
        return Err(TypeError::SymbolShouldBeStruct { span });
    };

    let def_fields = def_fields
        .iter()
        .map(|(field_sym, field_typ)| (field_sym.inner, (field_sym.meta, field_typ.clone())))
        .collect::<HashMap<_, _>>();

    // Set to keep track of fields in the struct constructor. Used to make sure no duplicates occur.
    let mut seen_fields = HashSet::new();

    let fields = fields
        .into_iter()
        .map(|(field_sym, field_bnd)| {
            let field_bnd = expr::constrain_expr(field_bnd, env)?;

            expect(
                seen_fields.insert(field_sym.inner),
                TypeError::ConstructDuplicateField {
                    sym: field_sym.to_string(),
                    span: field_sym.meta,
                },
            )?;

            let Some((def_span, def_typ)) = def_fields.get(field_sym.inner) else {
                return Err(TypeError::UnknownStructField {
                    sym: field_sym.to_string(),
                    span: field_sym.meta,
                });
            };

            env.uf.expect_type(
                field_bnd.meta.index,
                def_typ.clone(),
                |field_typ, def_typ| TypeError::MismatchedStructField {
                    expect: def_typ,
                    got: field_typ,
                    span_expected: *def_span,
                    span_got: field_sym.meta,
                },
            )?;

            Ok((field_sym, field_bnd))
        })
        .collect::<Result<Vec<_>, _>>()?;

    // Verify that all fields from the struct definition are present.
    for (def_sym, (def_span, _)) in def_fields {
        expect(
            seen_fields.contains(def_sym),
            TypeError::ConstructMissingField {
                sym: def_sym.to_string(),
                struct_span: sym.meta,
                def_span,
            },
        )?;
    }

    let index = env.uf.add(PartialType::Var { sym: sym.inner });

    Ok(Constrained {
        meta: MetaConstrained { span, index },
        inner: ExprConstrained::Struct { sym, fields },
    })
}
