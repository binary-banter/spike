pub mod reveal_functions;

use crate::passes::parse::{Def, Expr, Lit, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgRevealed<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<'p, UniqueSym<'p>, RExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum RExpr<'p> {
    Lit {
        val: Lit,
    },
    Var {
        sym: UniqueSym<'p>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Prim {
        op: Op,
        args: Vec<RExpr<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        bdy: Box<RExpr<'p>>,
    },
    If {
        cnd: Box<RExpr<'p>>,
        thn: Box<RExpr<'p>>,
        els: Box<RExpr<'p>>,
    },
    Apply {
        fun: Box<RExpr<'p>>,
        args: Vec<RExpr<'p>>,
    },
    Loop {
        bdy: Box<RExpr<'p>>,
    },
    Break {
        bdy: Box<RExpr<'p>>,
    },
    Return {
        bdy: Box<RExpr<'p>>,
    },
    Continue,
    Seq {
        stmt: Box<RExpr<'p>>,
        cnt: Box<RExpr<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
    },
    Struct { sym: UniqueSym<'p>, fields: Vec<(&'p str, RExpr<'p>)> },
    AccessField { strct: Box<RExpr<'p>>, field: &'p str },
}

impl<'p> From<PrgRevealed<'p>> for PrgUniquified<'p> {
    fn from(value: PrgRevealed<'p>) -> Self {
        PrgUniquified {
            defs: value
                .defs
                .into_iter()
                .map(|(sym, def)| (sym, def.into()))
                .collect(),
            entry: value.entry,
        }
    }
}

impl<'p> From<Def<'p, UniqueSym<'p>, RExpr<'p>>> for Def<'p, UniqueSym<'p>, Expr<'p, UniqueSym<'p>>> {
    fn from(value: Def<'p, UniqueSym<'p>, RExpr<'p>>) -> Self {
        match value {
            Def::Fn {
                sym,
                params,
                typ,
                bdy,
            } => Def::Fn {
                sym,
                params,
                typ,
                bdy: bdy.into(),
            },
            Def::Struct { sym, fields } => Def::Struct {
                sym,
                fields
            },
            Def::Enum { sym, variants } => Def::Enum { sym, variants },
        }
    }
}

impl<'p> From<RExpr<'p>> for Expr<'p, UniqueSym<'p>> {
    fn from(value: RExpr<'p>) -> Self {
        match value {
            RExpr::Lit { val } => Expr::Lit { val },
            RExpr::Prim { op, args } => Expr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
            },
            RExpr::Let { sym, bnd, bdy } => Expr::Let {
                sym,
                mutable: true,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
            },
            RExpr::If { cnd, thn, els } => Expr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
            },
            RExpr::Apply { fun, args } => Expr::Apply {
                fun: Box::new((*fun).into()),
                args: args.into_iter().map(Into::into).collect(),
            },
            RExpr::Var { sym } | RExpr::FunRef { sym } => Expr::Var { sym },
            RExpr::Loop { bdy } => Expr::Loop {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Break { bdy } => Expr::Break {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Seq { stmt, cnt } => Expr::Seq {
                stmt: Box::new((*stmt).into()),
                cnt: Box::new((*cnt).into()),
            },
            RExpr::Assign { sym, bnd } => Expr::Assign {
                sym,
                bnd: Box::new((*bnd).into()),
            },
            RExpr::Continue => Expr::Continue,
            RExpr::Return { bdy } => Expr::Return {
                bdy: Box::new((*bdy).into()),
            },
            RExpr::Struct { sym, fields } => Expr::Struct {
                sym,
                fields: fields.into_iter().map(|(sym, expr)| (sym, expr.into())).collect()
            },
            RExpr::AccessField { strct, field } => Expr::AccessField {
                strct: Box::new((*strct).into()),
                field,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::passes::uniquify::PrgUniquified;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn reveal([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let uniquified_program: PrgUniquified =
            program.type_check().unwrap().uniquify().reveal().into();
        let mut io = TestIO::new(input);
        let result = uniquified_program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as reveal => reveal }
}
