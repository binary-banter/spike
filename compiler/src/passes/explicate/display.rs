use crate::passes::explicate::{CExpr, CTail};
use indenter::indented;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl Display for CTail<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CTail::Return { expr } => {
                write!(indented(f), "{expr}")
            }
            CTail::Seq { sym, bnd, tail } => {
                writeln!(indented(f), "{sym} = {bnd};")?;
                write!(f, "{tail}")
            }
            CTail::IfStmt { cnd, thn, els } => {
                write!(indented(f), "if {cnd} {{ jmp {thn} }} else {{ jmp {els} }}")
            }
            CTail::Goto { lbl } => {
                write!(indented(f), "jmp {lbl}")
            }
        }
    }
}

impl Display for CExpr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CExpr::Atom { atm } => {
                write!(f, "{atm}")
            }
            CExpr::BinaryOp {
                op,
                exprs: [lhs, rhs],
            } => {
                write!(f, "{lhs} {op} {rhs}")
            }
            CExpr::UnaryOp { op, expr } => {
                write!(f, "{op}{expr}")
            }
            CExpr::Apply { fun, args } => {
                write!(f, "{fun}({})", args.iter().map(|(arg, _)| arg).format(", "))
            }
            CExpr::FunRef { sym } => {
                write!(f, "{sym}")
            }
            CExpr::Struct { sym, fields } => {
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
            CExpr::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            CExpr::Asm { instrs } => {
                writeln!(f, "asm {{")?;
                for instr in instrs {
                    writeln!(indented(f), "{instr}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
