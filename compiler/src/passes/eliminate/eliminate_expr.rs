use crate::passes::eliminate::ExprEliminated;
use crate::passes::explicate::ExprExplicated;
use functor_derive::Functor;

pub fn eliminate_expr(e: ExprExplicated) -> ExprEliminated {
    match e {
        ExprExplicated::Atom { atm } => ExprEliminated::Atom { atm },
        ExprExplicated::BinaryOp { op, exprs } => ExprEliminated::BinaryOp { op, exprs },
        ExprExplicated::UnaryOp { op, expr } => ExprEliminated::UnaryOp { op, expr },
        ExprExplicated::Apply { fun, args } => ExprEliminated::Apply {
            fun,
            args: args.fmap(|(arg, _)| arg),
        },
        ExprExplicated::FunRef { sym } => ExprEliminated::FunRef { sym },
        ExprExplicated::Asm { instrs } => ExprEliminated::Asm { instrs },
        ExprExplicated::Struct { .. } | ExprExplicated::AccessField { .. } => unreachable!(),
    }
}
