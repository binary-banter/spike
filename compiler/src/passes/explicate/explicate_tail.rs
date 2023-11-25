use crate::passes::atomize::{AExpr, Atom};
use crate::passes::explicate::explicate::Env;
use crate::passes::explicate::explicate_assign::explicate_assign;
use crate::passes::explicate::CTail;
use crate::passes::parse::types::Type;
use crate::passes::parse::Meta;
use crate::utils::gen_sym::{gen_sym, UniqueSym};

pub fn explicate_tail<'p>(
    expr: Meta<Type<UniqueSym<'p>>, AExpr<'p>>,
    env: &mut Env<'_, 'p>,
) -> CTail<'p> {
    let tmp = gen_sym("return");
    let tail = CTail::Return {
        expr: Meta {
            meta: expr.meta.clone(),
            inner: Atom::Var { sym: tmp },
        },
    };
    explicate_assign(tmp, expr, tail, env)
}
