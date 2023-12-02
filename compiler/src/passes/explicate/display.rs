use crate::passes::explicate::{ExprExplicated, TailExplicated};
use indenter::indented;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl Display for TailExplicated<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TailExplicated::Return { expr } => {
                write!(indented(f), "{expr}")
            }
            TailExplicated::Seq { sym, bnd, tail } => {
                writeln!(indented(f), "{sym} = {bnd};")?;
                write!(f, "{tail}")
            }
            TailExplicated::IfStmt { cnd, thn, els } => {
                write!(indented(f), "if {cnd} {{ jmp {thn} }} else {{ jmp {els} }}")
            }
            TailExplicated::Goto { lbl } => {
                write!(indented(f), "jmp {lbl}")
            }
        }
    }
}

impl Display for ExprExplicated<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprExplicated::Atom { atm } => {
                write!(f, "{atm}")
            }
            ExprExplicated::BinaryOp {
                op,
                exprs: [lhs, rhs],
            } => {
                write!(f, "{lhs} {op} {rhs}")
            }
            ExprExplicated::UnaryOp { op, expr } => {
                write!(f, "{op}{expr}")
            }
            ExprExplicated::Apply { fun, args } => {
                write!(f, "{fun}({})", args.iter().map(|(arg, _)| arg).format(", "))
            }
            ExprExplicated::FunRef { sym } => {
                write!(f, "{sym}")
            }
            ExprExplicated::Struct { sym, fields } => {
                writeln!(f, "{sym} {{")?;
                writeln!(
                    indented(f),
                    "{}",
                    fields
                        .iter()
                        .map(|(sym, bnd)| format!("{sym}: {bnd},"))
                        .format("\n")
                )?;
                write!(f, "}}")
            }
            ExprExplicated::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            ExprExplicated::Asm { instrs } => {
                writeln!(f, "asm {{")?;
                for instr in instrs {
                    writeln!(indented(f), "{instr}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
