use crate::passes::parse::{Constrained, Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::error::TypeError::MismatchedAssignBinding;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};
use crate::utils::expect::expect;
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_assign<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    sym: Spanned<UniqueSym<'p>>,
    bnd: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let bnd = expr::constrain_expr(bnd, env)?;

    let EnvEntry::Type { mutable, typ } = env.scope[&sym.inner] else {
        return Err(TypeError::SymbolShouldBeVariable { span: sym.meta });
    };

    expect(mutable, TypeError::ModifyImmutable { span: sym.meta })?;

    env.uf
        .expect_equal(typ, bnd.meta.index, |sym_typ, bnd_type| {
            MismatchedAssignBinding {
                expect: sym_typ,
                got: bnd_type,
                span_expected: sym.meta,
                span_got: bnd.meta.span,
            }
        })?;

    let typ = env.uf.add(PartialType::Unit);

    Ok(Constrained {
        meta: MetaConstrained { span, index: typ },
        inner: ExprConstrained::Assign {
            sym,
            bnd: Box::new(bnd),
        },
    })
}
