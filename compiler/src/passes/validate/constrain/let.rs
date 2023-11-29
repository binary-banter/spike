use crate::passes::parse::types::Type;
use crate::passes::parse::{Constrained, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_let<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    sym: Spanned<UniqueSym<'p>>,
    mutable: bool,
    typ: Option<Type<Spanned<UniqueSym<'p>>>>,
    bnd: Spanned<ExprUniquified<'p>>,
    bdy: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let bnd = expr::constrain_expr(bnd, env)?;

    if let Some(typ) = &typ {
        env.uf.expect_type(bnd.meta.index, typ.clone(), |got, _| {
            TypeError::MismatchedLetBinding {
                got,
                span_expected: (0, 0), //TODO span of typ
                span_got: bnd.meta.span,
            }
        })?;
    }

    env.scope.insert(
        sym.inner,
        EnvEntry::Type {
            mutable,
            typ: bnd.meta.index,
        },
    );
    let bdy = expr::constrain_expr(bdy, env)?;

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: bdy.meta.index,
        },
        inner: ExprConstrained::Let {
            sym,
            mutable,
            typ,
            bnd: Box::new(bnd),
            bdy: Box::new(bdy),
        },
    })
}
