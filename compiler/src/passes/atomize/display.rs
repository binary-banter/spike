use crate::passes::atomize::AExpr;
use indenter::indented;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl Display for AExpr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AExpr::Atom { atm } => {
                write!(f, "{atm}")
            }
            AExpr::FunRef { sym } => {
                write!(f, "{sym}")
            }
            AExpr::UnaryOp { op, expr } => {
                write!(f, "({op}{expr})")
            }
            AExpr::BinaryOp {
                op,
                exprs: [e1, e2],
            } => {
                write!(f, "({e1} {op} {e2})")
            }
            AExpr::Let { sym, bnd, bdy } => {
                writeln!(f, "(let {sym} = {bnd};",)?;
                write!(f, "{bdy})")
            }
            AExpr::If { cnd, thn, els } => {
                writeln!(f, "if {cnd} {{")?;
                writeln!(indented(f), "{thn}")?;
                writeln!(f, "}} else {{")?;
                writeln!(indented(f), "{els}")?;
                write!(f, "}}")
            }
            AExpr::Apply { fun, args } => {
                write!(f, "{fun}({})", args.iter().map(|(atm, _)| atm).format(", "))
            }
            AExpr::Loop { bdy } => {
                writeln!(f, "loop {{")?;
                writeln!(indented(f), "{bdy}")?;
                write!(f, "}}")
            }
            AExpr::Break { bdy } => {
                write!(f, "break {bdy}")
            }
            AExpr::Continue => {
                write!(f, "continue")
            }
            AExpr::Return { bdy } => {
                write!(f, "return {bdy}")
            }
            AExpr::Seq { stmt, cnt } => {
                writeln!(f, "{stmt};")?;
                write!(f, "{cnt}")
            }
            AExpr::Assign { sym, bnd } => {
                write!(f, "{sym} = {bnd}")
            }
            AExpr::Struct { sym, fields } => {
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
            AExpr::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            AExpr::Asm { .. } => todo!(),
        }
    }
}
