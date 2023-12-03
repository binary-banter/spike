use crate::passes::assign::include_liveness::{handle_instr, ReadWriteOp};
use crate::passes::assign::{InterferenceGraph, LArg, LFun};
use crate::passes::select::VarArg;
use petgraph::graphmap::GraphMap;
use petgraph::Undirected;
use std::collections::HashMap;

impl<'p> LFun<'p> {
    #[must_use]
    pub fn compute_interference(&self) -> InterferenceGraph<'p> {
        let mut graph = GraphMap::<LArg<'p>, (), Undirected>::new();

        for block in self.blocks.values() {
            for (instr, live_after) in &block.instrs {
                //TODO move optimization: If instruction is a move instruction then for every in w in writes, if w != dst and v != src, add the edge (dst, w).
                handle_instr(instr, &HashMap::new(), |arg, op| {
                    let w = match (arg, op) {
                        (VarArg::Reg { reg }, ReadWriteOp::Write | ReadWriteOp::ReadWrite) => {
                            LArg::Reg { reg: *reg }
                        }
                        (VarArg::XVar { sym }, ReadWriteOp::Write | ReadWriteOp::ReadWrite) => {
                            LArg::Var { sym: *sym }
                        }
                        // In case a variable is only read but never written to, we still need to add it to the graph
                        (VarArg::XVar { sym }, ReadWriteOp::Read) => {
                            graph.add_node(LArg::Var { sym: *sym });
                            return;
                        }
                        _ => return,
                    };

                    graph.add_node(w);
                    for &l in live_after {
                        if w == l {
                            continue;
                        };
                        graph.add_edge(w, l, ());
                    }
                });
            }
        }

        InterferenceGraph(graph)
    }
}
