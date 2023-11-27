use crate::passes::parse::types::Type;
use crate::passes::parse::{Constrained, Meta, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_if<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    cnd: Spanned<ExprUniquified<'p>>,
    thn: Spanned<ExprUniquified<'p>>,
    els: Spanned<ExprUniquified<'p>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let cnd = expr::constrain_expr(cnd, env)?;

    env.uf.expect_type(cnd.meta.index, Type::Bool, |got, _| {
        TypeError::IfExpectBool {
            got,
            span_got: cnd.meta.span,
        }
    })?;

    let thn = expr::constrain_expr(thn, env)?;
    let els = expr::constrain_expr(els, env)?;

    let out_index = env
        .uf
        .expect_equal(thn.meta.index, els.meta.index, |thn_type, els_type| {
            TypeError::IfExpectEqual {
                thn: thn_type,
                els: els_type,
                span_thn: thn.meta.span,
                span_els: els.meta.span,
            }
        })?;

    Ok(Constrained {
        meta: MetaConstrained {
            span,
            index: out_index,
        },
        inner: ExprConstrained::If {
            cnd: Box::new(cnd),
            thn: Box::new(thn),
            els: Box::new(els),
        },
    })
}
