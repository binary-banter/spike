pub mod reveal_functions;

use crate::passes::parse::{Def, Expr, Lit, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::parse::types::Type;
use crate::passes::type_check::TExpr;

#[derive(Debug, PartialEq)]
pub struct PrgRevealed<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<'p, UniqueSym<'p>, RExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum RExpr<'p> {
    Lit {
        val: Lit,
        typ: Type<UniqueSym<'p>>,
    },
    Var {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: Op,
        args: Vec<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    If {
        cnd: Box<RExpr<'p>>,
        thn: Box<RExpr<'p>>,
        els: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Box<RExpr<'p>>,
        args: Vec<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Loop {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Break {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Return {
        bdy: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Continue{
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        stmt: Box<RExpr<'p>>,
        cnt: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<RExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, RExpr<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Box<RExpr<'p>>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
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

impl<'p> From<Def<'p, UniqueSym<'p>, RExpr<'p>>> for Def<'p, UniqueSym<'p>, TExpr<'p, UniqueSym<'p>>>
{
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
            Def::TypeDef { sym, def } => Def::TypeDef { sym, def },
        }
    }
}

impl<'p> From<RExpr<'p>> for TExpr<'p, UniqueSym<'p>> {
    fn from(value: RExpr<'p>) -> Self {
        match value {
            RExpr::Lit { val, typ } => TExpr::Lit { val, typ },
            RExpr::Prim { op, args, typ } => TExpr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
                typ,
            },
            RExpr::Let { sym, bnd, bdy, typ } => TExpr::Let {
                sym,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
                typ,
            },
            RExpr::If { cnd, thn, els, typ } => TExpr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
                typ,
            },
            RExpr::Apply { fun, args, typ } => TExpr::Apply {
                fun: Box::new((*fun).into()),
                args: args.into_iter().map(Into::into).collect(),
                typ,
            },
            RExpr::Var { sym, typ } | RExpr::FunRef { sym, typ } => TExpr::Var { sym , typ},
            RExpr::Loop { bdy, typ } => TExpr::Loop {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            RExpr::Break { bdy, typ } => TExpr::Break {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            RExpr::Seq { stmt, cnt, typ } => TExpr::Seq {
                stmt: Box::new((*stmt).into()),
                cnt: Box::new((*cnt).into()),
                typ,
            },
            RExpr::Assign { sym, bnd, typ } => TExpr::Assign {
                sym,
                bnd: Box::new((*bnd).into()),
                typ,
            },
            RExpr::Continue{typ} => TExpr::Continue{typ},
            RExpr::Return { bdy, typ } => TExpr::Return {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            RExpr::Struct { sym, fields, typ } => TExpr::Struct {
                sym,
                fields: fields
                    .into_iter()
                    .map(|(sym, expr)| (sym, expr.into()))
                    .collect(),
                typ,
            },
            RExpr::AccessField { strct, field, typ } => TExpr::AccessField {
                strct: Box::new((*strct).into()),
                field,
                typ
            },
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
