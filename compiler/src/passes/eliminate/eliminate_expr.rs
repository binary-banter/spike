use crate::passes::eliminate::EExpr;
use crate::passes::explicate::CExpr;
use functor_derive::Functor;

pub fn eliminate_expr(e: CExpr) -> EExpr {
    match e {
        CExpr::Atom { atm } => EExpr::Atom { atm },
        CExpr::BinaryOp { op, exprs } => EExpr::BinaryOp { op, exprs },
        CExpr::UnaryOp { op, expr } => EExpr::UnaryOp { op, expr },
        CExpr::Apply { fun, args } => EExpr::Apply {
            fun,
            args: args.fmap(|(arg, _)| arg),
        },
        CExpr::FunRef { sym } => EExpr::FunRef { sym },
        CExpr::Struct { .. } | CExpr::AccessField { .. } => unreachable!(),
    }
}
