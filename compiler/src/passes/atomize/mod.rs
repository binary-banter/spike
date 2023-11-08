pub mod atomize;

use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, Op};
use crate::passes::validate::{TExpr, TLit};
use crate::utils::gen_sym::UniqueSym;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct PrgAtomized<'p> {
    pub defs: HashMap<UniqueSym<'p>, Def<'p, UniqueSym<'p>, AExpr<'p>>>,
    pub entry: UniqueSym<'p>,
}

#[derive(Debug, PartialEq)]
pub enum AExpr<'p> {
    Atom {
        atm: Atom<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Prim {
        op: Op,
        args: Vec<Atom<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Let {
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
        bdy: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    If {
        cnd: Box<AExpr<'p>>,
        thn: Box<AExpr<'p>>,
        els: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Apply {
        fun: Atom<'p>,
        args: Vec<(Atom<'p>, Type<UniqueSym<'p>>)>,
        typ: Type<UniqueSym<'p>>,
    },
    FunRef {
        sym: UniqueSym<'p>,
        typ: Type<UniqueSym<'p>>,
    },
    Loop {
        bdy: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Break {
        bdy: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Continue {
        typ: Type<UniqueSym<'p>>,
    },
    Seq {
        stmt: Box<AExpr<'p>>,
        cnt: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Assign {
        sym: UniqueSym<'p>,
        bnd: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Return {
        bdy: Box<AExpr<'p>>,
        typ: Type<UniqueSym<'p>>,
    },
    Struct {
        sym: UniqueSym<'p>,
        fields: Vec<(&'p str, Atom<'p>)>,
        typ: Type<UniqueSym<'p>>,
    },
    AccessField {
        strct: Atom<'p>,
        field: &'p str,
        typ: Type<UniqueSym<'p>>,
    },
}

impl<'p> AExpr<'p> {
    pub fn typ(&self) -> &Type<UniqueSym<'p>> {
        match self {
            AExpr::Atom { typ, .. }
            | AExpr::Prim { typ, .. }
            | AExpr::Let { typ, .. }
            | AExpr::If { typ, .. }
            | AExpr::Apply { typ, .. }
            | AExpr::Loop { typ, .. }
            | AExpr::Break { typ, .. }
            | AExpr::Continue { typ, .. }
            | AExpr::Return { typ, .. }
            | AExpr::Seq { typ, .. }
            | AExpr::Assign { typ, .. }
            | AExpr::Struct { typ, .. }
            | AExpr::AccessField { typ, .. }
            | AExpr::FunRef { typ, .. } => typ,
        }
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Atom<'p> {
    Val { val: TLit },
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

// TODO functor time
impl<'p> From<Def<'p, UniqueSym<'p>, AExpr<'p>>>
    for Def<'p, UniqueSym<'p>, TExpr<'p, UniqueSym<'p>>>
{
    fn from(value: Def<'p, UniqueSym<'p>, AExpr<'p>>) -> Self {
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

impl<'p> From<AExpr<'p>> for TExpr<'p, UniqueSym<'p>> {
    fn from(value: AExpr<'p>) -> Self {
        match value {
            AExpr::Atom { atm, .. } => atm.into(),
            AExpr::Prim { op, args, typ } => TExpr::Prim {
                op,
                args: args.into_iter().map(Into::into).collect(),
                typ,
            },
            AExpr::Let { sym, bnd, bdy, typ } => TExpr::Let {
                sym,
                bnd: Box::new((*bnd).into()),
                bdy: Box::new((*bdy).into()),
                typ,
            },
            AExpr::If { cnd, thn, els, typ } => TExpr::If {
                cnd: Box::new((*cnd).into()),
                thn: Box::new((*thn).into()),
                els: Box::new((*els).into()),
                typ,
            },
            AExpr::Apply { fun, args, typ, .. } => TExpr::Apply {
                fun: Box::new(fun.into()),
                args: args.into_iter().map(|(a, _)| a.into()).collect(),
                typ,
            },
            AExpr::FunRef { sym, typ } => TExpr::Var { sym, typ },
            AExpr::Loop { bdy, typ } => TExpr::Loop {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            AExpr::Break { bdy, typ } => TExpr::Break {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            AExpr::Seq { stmt, cnt, typ } => TExpr::Seq {
                stmt: Box::new((*stmt).into()),
                cnt: Box::new((*cnt).into()),
                typ,
            },
            AExpr::Assign { sym, bnd, typ } => TExpr::Assign {
                sym,
                bnd: Box::new((*bnd).into()),
                typ,
            },
            AExpr::Continue { typ } => TExpr::Continue { typ },
            AExpr::Return { bdy, typ } => TExpr::Return {
                bdy: Box::new((*bdy).into()),
                typ,
            },
            AExpr::Struct { sym, fields, typ } => TExpr::Struct {
                sym,
                fields: fields
                    .into_iter()
                    .map(|(sym, atm)| (sym, atm.into()))
                    .collect(),
                typ,
            },
            AExpr::AccessField { strct, field, typ } => TExpr::AccessField {
                strct: Box::new(strct.into()),
                field,
                typ,
            },
        }
    }
}

// Note that casting to Never here is safe because this `From` is only used by the interpreter which doesn't care about the type information
impl<'p> From<Atom<'p>> for TExpr<'p, UniqueSym<'p>> {
    fn from(value: Atom<'p>) -> Self {
        match value {
            Atom::Val { val } => TExpr::Lit {
                val,
                typ: Type::Never,
            },
            Atom::Var { sym } => TExpr::Var {
                sym,
                typ: Type::Never,
            },
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::interpreter::TestIO;
//     use crate::utils::split_test::split_test;
//     use test_each_file::test_each_file;
//
//     fn atomize([test]: [&str; 1]) {
//         let (input, expected_output, expected_return, program) = split_test(test);
//         let program: PrgUniquified = program
//             .validate()
//             .unwrap()
//             .uniquify()
//             .reveal()
//             .atomize()
//             .into();
//         let mut io = TestIO::new(input);
//         let result = program.interpret(&mut io);
//
//         assert_eq!(result, expected_return.into(), "Incorrect program result.");
//         assert_eq!(io.outputs(), &expected_output, "Incorrect program output.");
//     }
//
//     test_each_file! { for ["test"] in "./programs/good" as atomize => atomize }
// }
