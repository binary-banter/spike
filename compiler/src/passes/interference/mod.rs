use crate::passes::select::io::Std;
use crate::passes::select::{Block, Instr, Reg, VarArg, X86Selected};
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use petgraph::graphmap::GraphMap;
use petgraph::Undirected;
use std::collections::{HashMap, HashSet};

pub mod assign_homes;
pub mod coloring_interference;
pub mod compute_interference;
pub mod liveness_analysis;

#[derive(Debug, PartialEq)]
pub struct X86Assigned<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, Arg>>,
    pub entry: UniqueSym<'p>,
    pub stack_space: usize,
    pub std: Std<'p>,
}

#[derive(Debug, PartialEq)]
pub struct LX86VarProgram<'p> {
    pub blocks: HashMap<UniqueSym<'p>, LBlock<'p>>,
    pub entry: UniqueSym<'p>,
    pub std: Std<'p>,
}

#[derive(Debug)]
pub struct X86WithInterference<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub interference: InterferenceGraph<'p>,
    pub std: Std<'p>,
}

#[derive(Debug)]
pub struct X86Colored<'p> {
    pub blocks: HashMap<UniqueSym<'p>, Block<'p, VarArg<'p>>>,
    pub entry: UniqueSym<'p>,
    pub color_map: HashMap<UniqueSym<'p>, Arg>,
    pub stack_space: usize,
    pub std: Std<'p>,
}

pub type InterferenceGraph<'p> = GraphMap<LArg<'p>, (), Undirected>;

#[derive(Debug, PartialEq)]
pub struct LBlock<'p> {
    pub instrs: Vec<(Instr<'p, VarArg<'p>>, HashSet<LArg<'p>>)>,
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Display)]
pub enum Arg {
    #[display(fmt = "${val}")]
    Imm { val: i64 },
    #[display(fmt = "%{reg}")]
    Reg { reg: Reg },
    #[display(fmt = "[%{reg} + ${off}]")]
    Deref { reg: Reg, off: i64 },
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Ord, PartialOrd)]
pub enum LArg<'p> {
    Var { sym: UniqueSym<'p> },
    Reg { reg: Reg },
}

impl<'p> From<LArg<'p>> for VarArg<'p> {
    fn from(val: LArg<'p>) -> Self {
        match val {
            LArg::Var { sym } => VarArg::XVar { sym },
            LArg::Reg { reg } => VarArg::Reg { reg },
        }
    }
}

impl<'p> From<LBlock<'p>> for Block<'p, VarArg<'p>> {
    fn from(value: LBlock<'p>) -> Self {
        Block {
            instrs: value.instrs.into_iter().map(|(instr, _)| instr).collect(),
        }
    }
}

impl<'p> From<Arg> for VarArg<'p> {
    fn from(value: Arg) -> Self {
        match value {
            Arg::Imm { val } => VarArg::Imm { val },
            Arg::Reg { reg } => VarArg::Reg { reg },
            Arg::Deref { reg, off } => VarArg::Deref { reg, off },
        }
    }
}

impl<'p> From<X86Assigned<'p>> for X86Selected<'p> {
    fn from(value: X86Assigned<'p>) -> Self {
        X86Selected {
            blocks: value.blocks.fmap(|v| v.fmap(Into::into)),
            entry: value.entry,
            std: value.std,
        }
    }
}