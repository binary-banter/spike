use crate::passes::parse::{Meta, Span};
use crate::passes::parse::types::Type;
use crate::passes::validate::{CMeta, ExprConstrained, ExprUniquified};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::constrain::uncover_globals::{Env, EnvEntry};
use crate::utils::gen_sym::UniqueSym;

pub fn constrain_let<'p>(env: &mut Env<'_, 'p>, span: Span, sym: Meta<Span, UniqueSym<'p>>, mutable: bool, typ: Option<Type<Meta<Span, UniqueSym<'p>>>>, bnd: Box<Meta<Span, ExprUniquified<'p>>>, bdy: Box<Meta<Span, ExprUniquified<'p>>>) -> Result<Meta<CMeta, ExprConstrained<'p>>, TypeError> {
    let bnd = expr::constrain_expr(*bnd, env)?;

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
    let bdy = expr::constrain_expr(*bdy, env)?;

    Ok(Meta {
        meta: CMeta {
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
