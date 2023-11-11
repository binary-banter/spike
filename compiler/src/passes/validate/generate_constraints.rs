use crate::passes::parse::{DefUniquified, ExprUniquified, Meta, Span};
use crate::passes::validate::{CMeta, DefConstrained, ExprConstrained, PrgConstrained};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::uncover_globals::uncover_globals;
use crate::passes::validate::uniquify::PrgUniquified;

pub struct GraphThingy {}

impl<'p> PrgUniquified<'p> {
    pub fn constrain(self) -> Result<PrgConstrained<'p>, TypeError> {
        let mut scope = uncover_globals(&self);

        Ok(PrgConstrained {
            defs: self.defs.into_iter().map(|def| {
                constrain_def(def).map(|def| (def.sym().inner, def))
            }).collect::<Result<_, _>>()?,
            entry: self.entry,
        })
    }
}

fn constrain_def(def: DefUniquified) -> Result<DefConstrained, TypeError> {
    let def = match def {
        DefUniquified::Fn { sym, params, typ, bdy } => DefConstrained::Fn{
            sym,
            params,
            typ,
            bdy: constrain_expr(bdy)?,
        },
        DefUniquified::TypeDef { sym, def } => DefConstrained::TypeDef { sym, def },
    };

    Ok(def)
}

fn constrain_expr(expr: Meta<Span, ExprUniquified>) -> Result<Meta<CMeta, ExprConstrained>, TypeError> {
    todo!()
}