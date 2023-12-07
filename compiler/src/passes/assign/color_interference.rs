use crate::passes::assign::{Arg, InterferenceGraph, LArg};
use crate::passes::select::{Reg, CALLEE_SAVED_NO_STACK};
use crate::utils::gen_sym::UniqueSym;
use binary_heap_plus::BinaryHeap;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

impl<'p> InterferenceGraph<'p> {
    #[must_use]
    pub fn color(self) -> (HashMap<UniqueSym<'p>, Arg>, usize) {
        let graph = self.0;
        let node_map = RefCell::new(HashMap::<LArg, isize>::new());
        let mut queue = BinaryHeap::new_by_key(|node| {
            graph
                .neighbors(*node)
                .filter(|nb| node_map.borrow().contains_key(nb))
                .count()
        });

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
                    node_map.borrow_mut().insert(node, node_weight);
                }
            }
        }

        while let Some(node) = queue.pop() {
            let used_colors = graph
                .neighbors(node)
                .filter(|nb| node_map.borrow().contains_key(nb))
                .map(|nb| node_map.borrow()[&nb])
                .collect::<HashSet<_>>();

            let chosen_color = (0..)
                .find(|i| !used_colors.contains(i))
                .expect("there are infinite numbers, lol");

            node_map.borrow_mut().insert(node, chosen_color);
        }

        let used_vars = node_map
            .borrow()
            .values()
            .filter(|&&n| n >= 10)
            .map(|&n| n - 10)
            .max()
            .unwrap_or_default() as usize;

        let stack_space = (8 * used_vars).div_ceil(16) * 16;

        let colors = node_map
            .take()
            .into_iter()
            .filter_map(|(node, color)| match node {
                LArg::Var { sym } => Some((sym, arg_from_color(color))),
                LArg::Reg { .. } => None,
            })
            .collect();

        (colors, stack_space)
    }
}

fn arg_from_color(i: isize) -> Arg {
    match i {
        -5 => Arg::Reg(Reg::R15),
        -4 => Arg::Reg(Reg::R11),
        -3 => Arg::Reg(Reg::RBP),
        -2 => Arg::Reg(Reg::RSP),
        -1 => Arg::Reg(Reg::RAX),
        0 => Arg::Reg(Reg::RCX),
        1 => Arg::Reg(Reg::RDX),
        2 => Arg::Reg(Reg::RSI),
        3 => Arg::Reg(Reg::RDI),
        4 => Arg::Reg(Reg::R8),
        5 => Arg::Reg(Reg::R9),
        6 => Arg::Reg(Reg::R10),
        7 => Arg::Reg(Reg::RBX),
        8 => Arg::Reg(Reg::R12),
        9 => Arg::Reg(Reg::R13),
        10 => Arg::Reg(Reg::R14),
        i => {
            assert!(
                i > 10,
                "Something went wrong while coloring the assign graph."
            );
            Arg::Deref {
                reg: Reg::RBP,
                off: -8 * ((i - 10) as i64 + CALLEE_SAVED_NO_STACK.len() as i64),
            }
        }
    }
}
