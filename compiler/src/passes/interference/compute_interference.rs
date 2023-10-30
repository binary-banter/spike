use crate::passes::interference::liveness_analysis::{handle_instr, ReadWriteOp};
use crate::passes::interference::{InterferenceGraph, LArg, LX86VarProgram, X86WithInterference};
use crate::passes::select::VarArg;
use std::collections::HashMap;

impl<'p> LX86VarProgram<'p> {
    #[must_use]
    pub fn compute_interference(self) -> X86WithInterference<'p> {
        X86WithInterference {
            interference: self.build_graph(),
            blocks: self
                .blocks
                .into_iter()
                .map(|(name, block)| (name, block.into()))
                .collect(),
            entry: self.entry,
            std: self.std,
        }
    }

    fn build_graph(&self) -> InterferenceGraph<'p> {
        let mut graph = InterferenceGraph::new();

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

        graph
    }
}
