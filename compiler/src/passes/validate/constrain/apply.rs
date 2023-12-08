use crate::passes::parse::{Constrained, Span, Spanned};
use crate::passes::validate::constrain::expr;
use crate::passes::validate::constrain::uncover_globals::Env;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::{ExprConstrained, ExprUniquified, MetaConstrained};

pub fn constrain_apply<'p>(
    env: &mut Env<'_, 'p>,
    span: Span,
    fun: Spanned<ExprUniquified<'p>>,
    args: Vec<Spanned<ExprUniquified<'p>>>,
) -> Result<Constrained<ExprConstrained<'p>>, TypeError> {
    let fun = expr::constrain_expr(fun, env)?;
    let args: Vec<_> = args
        .into_iter()
        .map(|arg| expr::constrain_expr(arg, env))
        .collect::<Result<_, _>>()?;

    let p_typ = env.uf.get(fun.meta.index).clone();
    let PartialType::Fn { params, typ } = p_typ else {
        return Err(TypeError::MismatchedExpectFn {
            got: p_typ.to_string(env.uf),
            span_got: fun.meta.span,
        });
    };

    if params.len() != args.len() {
        return Err(TypeError::ArgCountMismatch {
            got: args.len(),
            expected: params.len(),
            span, // todo: maybe highlight only the args and params?
        });
    }

    for (arg, param_type) in args.iter().zip(params.iter()) {
        env.uf
            .expect_equal(arg.meta.index, *param_type, |arg_type, param_type| {
                TypeError::FnArgExpect {
                    arg: arg_type,
                    param: param_type,
                    span_arg: arg.meta.span,
                }
            })?;
    }

    Ok(Constrained {
        meta: MetaConstrained { span, index: typ },
        inner: ExprConstrained::Apply {
            fun: Box::new(fun),
            args,
        },
    })
}
