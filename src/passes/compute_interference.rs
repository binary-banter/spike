// use crate::language::x86var::{IX86VarProgram, InterferenceGraph, LX86VarProgram};
// use crate::passes::liveness_analysis::instr_writes;
//
// impl<'p> LX86VarProgram<'p> {
//     pub fn compute_interference(self) -> IX86VarProgram<'p> {
//         IX86VarProgram {
//             interference: self.build_graph(),
//             entry: self.entry,
//             blocks: self
//                 .blocks
//                 .into_iter()
//                 .map(|(name, block)| (name, block.into()))
//                 .collect(),
//         }
//     }
//
//     fn build_graph(&self) -> InterferenceGraph<'p> {
//         let mut graph = InterferenceGraph::new();
//
//         for block in self.blocks.values() {
//             for (instr, live_after) in &block.instrs {
//                 //TODO move optimization: If instruction is a move instruction then for every in w in writes, if w != dst and v != src, add the edge (dst, w).
//                 for w in instr_writes(instr) {
//                     graph.add_node(w);
//                     for &l in live_after {
//                         if w == l {
//                             continue;
//                         };
//                         graph.add_edge(w, l, ());
//                     }
//                 }
//             }
//         }
//
//         graph
//     }
// }
