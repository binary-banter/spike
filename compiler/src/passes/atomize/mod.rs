pub mod atomize;

use crate::passes::parse::{Def, Expr, Lit, Op};
use crate::passes::uniquify::PrgUniquified;
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;
use crate::passes::type_check::TExpr;

#[derive(Debug, PartialEq)]
pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<'p, UniqueSym<'p>, AExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum AExpr<'p> {
    Atom {
        atm: Atom<'p>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
        bdy: Box<AExpr<'p>>,
    },
    If {
        cnd: Box<AExpr<'p>>,
        thn: Box<AExpr<'p>>,
        els: Box<AExpr<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<Atom<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
    },
    Loop {
        bdy: Box<AExpr<'p>>,
    },
    Break {
        bdy: Box<AExpr<'p>>,
    },
    Continue,
    Seq {
        stmt: Box<AExpr<'p>>,
        cnt: Box<AExpr<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
    },
    Return {
        bdy: Box<AExpr<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Atom<'p>)>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
    },
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Atom<'p> {
    Val { val: Lit },
    Var { sym: UniqueSym<'p> },
}

impl<'p> Atom<'p> {
    pub fn var(self) -> UniqueSym<'p> {
        if let Atom::Var { sym } = self {
            sym
        } else {
            panic!()
        }
    }
}

impl<'p> From<PrgAtomized<'p>> for PrgUniquified<'p> {
    fn from(value: PrgAtomized<'p>) -> Self {
        todo!();
        // PrgUniquified {
        //     defs: value
        //         .defs
        //         .into_iter()
        //         .map(|(sym, def)| (sym, def.into()))
        //         .collect(),
        //     entry: value.entry,
        // }
    }
}

// TODO functor time
impl<'p> From<Def<'p, UniqueSym<'p>, AExpr<'p>>>
    for Def<'p, UniqueSym<'p>, Expr<'p, UniqueSym<'p>>>
{
    fn from(value: Def<'p, UniqueSym<'p>, AExpr<'p>>) -> Self {
        todo!()
        // match value {
        //     Def::Fn {
        //         sym,
        //         params,
        //         typ,
        //         bdy,
        //     } => Def::Fn {
        //         sym,
        //         params,
        //         typ,
        //         bdy: bdy.into(),
        //     },
        //     Def::Struct { sym, fields } => Def::Struct { sym, fields },
        //     Def::Enum { sym, variants } => Def::Enum { sym, variants },
        // }
    }
}

impl<'p> From<AExpr<'p>> for TExpr<'p, UniqueSym<'p>> {
    fn from(value: AExpr<'p>) -> Self {
        todo!();
        // match value {
        //     AExpr::Atom { atm } => atm.into(),
        //     AExpr::Prim { op, args } => TExpr::Prim {
        //         op,
        //         args: args.into_iter().map(Into::into).collect(),
        //     },
        //     AExpr::Let { sym, bnd, bdy } => TExpr::Let {
        //         sym,
        //         mutable: true,
        //         bnd: Box::new((*bnd).into()),
        //         bdy: Box::new((*bdy).into()),
        //     },
        //     AExpr::If { cnd, thn, els } => TExpr::If {
        //         cnd: Box::new((*cnd).into()),
        //         thn: Box::new((*thn).into()),
        //         els: Box::new((*els).into()),
        //     },
        //     AExpr::Apply { fun, args } => TExpr::Apply {
        //         fun: Box::new(fun.into()),
        //         args: args.into_iter().map(Into::into).collect(),
        //     },
        //     AExpr::FunRef { sym } => TExpr::Var { sym },
        //     AExpr::Loop { bdy } => TExpr::Loop {
        //         bdy: Box::new((*bdy).into()),
        //     },
        //     AExpr::Break { bdy } => TExpr::Break {
        //         bdy: Box::new((*bdy).into()),
        //     },
        //     AExpr::Seq { stmt, cnt } => TExpr::Seq {
        //         stmt: Box::new((*stmt).into()),
        //         cnt: Box::new((*cnt).into()),
        //     },
        //     AExpr::Assign { sym, bnd } => TExpr::Assign {
        //         sym,
        //         bnd: Box::new((*bnd).into()),
        //     },
        //     AExpr::Continue => TExpr::Continue,
        //     AExpr::Return { bdy } => TExpr::Return {
        //         bdy: Box::new((*bdy).into()),
        //     },
        //     AExpr::Struct { sym, fields } => TExpr::Struct {
        //         sym,
        //         fields: fields
        //             .into_iter()
        //             .map(|(sym, atm)| (sym, atm.into()))
        //             .collect(),
        //     },
        //     AExpr::AccessField { strct, field } => TExpr::AccessField {
        //         strct: Box::new(strct.into()),
        //         field,
        //     },
        // }
    }
}

impl<'p> From<Atom<'p>> for Expr<'p, UniqueSym<'p>> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Val { val } => Expr::Lit { val },
            Atom::Var { sym } => Expr::Var { sym },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::TestIO;
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;
    use crate::passes::uniquify::PrgUniquified;

    fn atomize([test]: [&str; 1]) {
        let (input, expected_output, expected_return, program) = split_test(test);
        let program: PrgUniquified = program
            .type_check()
            .unwrap()
            .uniquify()
            .reveal()
            .atomize()
            .into();
        let mut io = TestIO::new(input);
        let result = program.interpret(&mut io);

        assert_eq!(result, expected_return.into(), "Incorrect program result.");
        assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
    }

    test_each_file! { for ["test"] in "./programs/good" as atomize => atomize }
}
