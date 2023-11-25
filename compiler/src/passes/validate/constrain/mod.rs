use crate::passes::parse::types::Type;
use crate::passes::parse::{Meta, Span};
use crate::passes::validate::constrain::def::constrain_def;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::partial_type::PartialType;
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::{partial_type, PrgConstrained};
use crate::utils::gen_sym::UniqueSym;
use crate::utils::union_find::{UnionFind, UnionIndex};
use uncover_globals::uncover_globals;

mod access_field;
mod apply;
mod assign;
mod binary_op;
mod r#break;
mod r#continue;
pub mod def;
pub mod expr;
mod r#if;
mod r#let;
mod lit;
mod r#loop;
mod r#return;
mod seq;
mod r#struct;
mod unary_op;
mod uncover_globals;
mod var;

impl<'p> PrgUniquified<'p> {
    pub fn constrain(self) -> Result<PrgConstrained<'p>, TypeError> {
        let mut uf = UnionFind::new();
        let mut scope = uncover_globals(&self, &mut uf)?;

        Ok(PrgConstrained {
            defs: self
                .defs
                .into_iter()
                .map(|def| {
                    constrain_def(def, &mut scope, &mut uf).map(|def| (def.sym().inner, def))
                })
                .collect::<Result<_, _>>()?,
            entry: self.entry,
            uf,
            std: self.std,
        })
    }
}

impl<'p> UnionFind<PartialType<'p>> {
    pub fn expect_equal(
        &mut self,
        a: UnionIndex,
        b: UnionIndex,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        self.try_union_by(a, b, partial_type::combine_partial_types)
            .map_err(|_| {
                let typ_a = self.get(a).clone();
                let str_a = typ_a.to_string(self);
                let typ_b = self.get(b).clone();
                let str_b = typ_b.to_string(self);
                map_err(str_a, str_b)
            })
    }

    pub fn expect_type(
        &mut self,
        a: UnionIndex,
        t: Type<Meta<Span, UniqueSym<'p>>>,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        let t_index = self.type_to_index(t);
        self.expect_equal(a, t_index, map_err)
    }

    pub fn expect_partial_type(
        &mut self,
        a: UnionIndex,
        t: PartialType<'p>,
        map_err: impl FnOnce(String, String) -> TypeError,
    ) -> Result<UnionIndex, TypeError> {
        let t_index = self.add(t);
        self.expect_equal(a, t_index, map_err)
    }

    pub fn type_to_index(&mut self, t: Type<Meta<Span, UniqueSym<'p>>>) -> UnionIndex {
        let pt = match t {
            Type::I64 => PartialType::I64,
            Type::U64 => PartialType::U64,
            Type::Bool => PartialType::Bool,
            Type::Unit => PartialType::Unit,
            Type::Never => PartialType::Never,
            Type::Fn { params, typ } => PartialType::Fn {
                params: params
                    .into_iter()
                    .map(|param| self.type_to_index(param))
                    .collect(),
                typ: self.type_to_index(*typ),
            },
            Type::Var { sym } => PartialType::Var { sym: sym.inner },
        };

        self.add(pt)
    }
}
