use crate::passes::parse::types::Type;
use crate::passes::parse::{Def, TypeDef};
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgConstrained;
use crate::utils::gen_sym::UniqueSym;
use petgraph::algo::toposort;
use petgraph::prelude::GraphMap;
use petgraph::Directed;

impl<'p> PrgConstrained<'p> {
    pub fn check_sized(&self) -> Result<(), TypeError> {
        let mut size_graph: GraphMap<UniqueSym<'p>, (), Directed> = GraphMap::new();
        for def in self.defs.values() {
            #[allow(clippy::single_match)]
            match def {
                Def::TypeDef { sym, def } => match def {
                    TypeDef::Struct { fields } => {
                        for (_, field) in fields {
                            match field {
                                Type::I64
                                | Type::U64
                                | Type::Bool
                                | Type::Unit
                                | Type::Never
                                | Type::Fn { .. } => {}
                                Type::Var { sym: field_sym } => {
                                    size_graph.add_edge(sym.inner, field_sym.inner, ());
                                }
                            }
                        }
                    }
                    TypeDef::Enum { .. } => todo!(),
                },
                _ => {}
            }
        }

        match toposort(&size_graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => Err(TypeError::UnsizedType {
                sym: cycle.node_id().to_string(),
                span: self.defs[&cycle.node_id()].sym().meta,
            }),
        }
    }
}
