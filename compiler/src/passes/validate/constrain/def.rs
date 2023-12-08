use crate::passes::parse::{Def, Spanned};
use crate::passes::validate::constrain::expr::constrain_expr;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{DefConstrained, DefUniquified};
use crate::utils::union_find::UnionFind;
use crate::utils::unique_sym::UniqueSym;
use std::collections::HashMap;

pub fn constrain_def<'p>(
    def: DefUniquified<'p>,
    scope: &mut HashMap<UniqueSym<'p>, EnvEntry<'p>>,
    uf: &mut UnionFind<PartialType<'p>>,
) -> Result<DefConstrained<'p>, TypeError> {
    let def = match def {
        Def::Fn {
            sym,
            params,
            typ,
            bdy,
        } => {
            // Put function parameters in scope.
            scope.extend(params.iter().map(|p| {
                (
                    p.sym.inner,
                    EnvEntry::Type {
                        mutable: p.mutable,
                        typ: uf.type_to_index(p.typ.clone()),
                    },
                )
            }));

            // Add return type to env and keep it for error handling.
            let return_index = uf.type_to_index(typ.clone());
            let mut env = Env {
                uf,
                scope,
                loop_type: None,
                return_type: &Spanned {
                    inner: return_index,
                    meta: sym.meta,
                }, // TODO replace sym.meta with return type index
            };

            // Constrain body of function.
            let bdy = constrain_expr(bdy, &mut env)?;

            // Return error if function body a type differs from its return type.
            uf.expect_equal(return_index, bdy.meta.index, |r, b| {
                TypeError::MismatchedFnReturn {
                    expect: r,
                    got: b,
                    span_expected: sym.meta,
                    span_got: bdy.meta.span,
                }
            })?;

            Def::Fn {
                sym,
                params,
                bdy,
                typ,
            }
        }
        Def::TypeDef { sym, def } => Def::TypeDef { sym, def },
    };

    Ok(def)
}
