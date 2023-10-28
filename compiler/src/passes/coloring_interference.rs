use crate::language::x86var::{Arg, InterferenceGraph, LArg, Reg, X86Colored, X86WithInterference};
use crate::utils::gen_sym::UniqueSym;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

impl<'p> X86WithInterference<'p> {
    pub fn color_interference(self) -> X86Colored<'p> {
        let (color_map, stack_space) = color_graph(self.interference);

        X86Colored {
            blocks: self.blocks,
            entry: self.entry,
            color_map,
            stack_space,
            std: self.std,
        }
    }
}

fn color_graph(graph: InterferenceGraph) -> (HashMap<UniqueSym, Arg>, usize) {
    let mut queue = Vec::new();
    let mut node_map = HashMap::<LArg, isize>::new();

    for node in graph.nodes() {
        match node {
            LArg::Var { .. } => {
                queue.push(node);
            }
            LArg::Reg { reg } => {
                let node_weight = match reg {
                    Reg::RCX => 0,
                    Reg::RDX => 1,
                    Reg::RSI => 2,
                    Reg::RDI => 3,
                    Reg::R8 => 4,
                    Reg::R9 => 5,
                    Reg::R10 => 6,
                    Reg::RBX => 7,
                    Reg::R12 => 8,
                    Reg::R13 => 9,
                    Reg::R14 => 10,
                    Reg::RAX => -1,
                    Reg::RSP => -2,
                    Reg::RBP => -3,
                    Reg::R11 => -4,
                    Reg::R15 => -5,
                };
                node_map.insert(node, node_weight);
            }
        }
    }

    while let Some(node) = queue.pop() {
        let used_colors = graph
            .neighbors(node)
            .filter_map(|nb| node_map.get(&nb))
            .collect::<HashSet<_>>();

        let chosen_color = (0..)
            .find(|i| !used_colors.contains(i))
            .expect("there are infinite numbers, lol");

        node_map.insert(node, chosen_color);

        queue.sort_by_key(|node| {
            graph
                .neighbors(*node)
                .filter_map(|nb| node_map.get(&nb))
                .unique()
                .count()
        });
    }

    let used_vars = node_map
        .values()
        .filter(|&&n| n >= 10)
        .map(|&n| n - 10)
        .max()
        .unwrap_or_default() as usize;

    let stack_space = (8 * used_vars).div_ceil(16) * 16;

    let colors = node_map
        .into_iter()
        .filter_map(|(node, color)| match node {
            LArg::Var { sym } => Some((sym, arg_from_color(color))),
            LArg::Reg { .. } => None,
        })
        .collect();

    (colors, stack_space)
}

fn arg_from_color(i: isize) -> Arg {
    match i {
        -5 => Arg::Reg { reg: Reg::R15 },
        -4 => Arg::Reg { reg: Reg::R11 },
        -3 => Arg::Reg { reg: Reg::RBP },
        -2 => Arg::Reg { reg: Reg::RSP },
        -1 => Arg::Reg { reg: Reg::RAX },
        0 => Arg::Reg { reg: Reg::RCX },
        1 => Arg::Reg { reg: Reg::RDX },
        2 => Arg::Reg { reg: Reg::RSI },
        3 => Arg::Reg { reg: Reg::RDI },
        4 => Arg::Reg { reg: Reg::R8 },
        5 => Arg::Reg { reg: Reg::R9 },
        6 => Arg::Reg { reg: Reg::R10 },
        7 => Arg::Reg { reg: Reg::RBX },
        8 => Arg::Reg { reg: Reg::R12 },
        9 => Arg::Reg { reg: Reg::R13 },
        10 => Arg::Reg { reg: Reg::R14 },
        i => {
            assert!(
                i > 10,
                "Something went wrong while coloring the interference graph."
            );
            Arg::Deref {
                reg: Reg::RBP,
                off: (-8 * (i - 10)) as i64,
            }
        }
    }
}
