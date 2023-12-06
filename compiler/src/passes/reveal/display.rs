use crate::passes::reveal::RExpr;
use indenter::indented;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl Display for RExpr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RExpr::Lit { val } => {
                write!(f, "{val}")
            }
            RExpr::Var { sym } => {
                write!(f, "{sym}")
            }
            RExpr::FunRef { sym } => {
                write!(f, "{sym}")
            }
            RExpr::UnaryOp { op, expr } => {
                write!(f, "({op}{expr})")
            }
            RExpr::BinaryOp {
                op,
                exprs: [e1, e2],
            } => {
                write!(f, "({e1} {op} {e2})")
            }
            RExpr::Let {
                sym,
                mutable,
                bnd,
                bdy,
            } => {
                let mutable = if *mutable { "mut " } else { "" };

                writeln!(f, "let {mutable}{sym} = {bnd};")?;
                write!(f, "{bdy}")
            }
            RExpr::If { cnd, thn, els } => {
                writeln!(f, "if {cnd} {{")?;
                writeln!(indented(f), "{thn}")?;
                writeln!(f, "}} else {{")?;
                writeln!(indented(f), "{els}")?;
                write!(f, "}}")
            }
            RExpr::Apply { fun, args } => {
                write!(f, "{fun}({})", args.iter().format(", "))
            }
            RExpr::Loop { bdy } => {
                writeln!(f, "loop {{")?;
                writeln!(indented(f), "{bdy}")?;
                write!(f, "}}")
            }
            RExpr::Break { bdy } => {
                write!(f, "break {bdy}")
            }
            RExpr::Continue => {
                write!(f, "continue")
            }
            RExpr::Return { bdy } => {
                write!(f, "return {bdy}")
            }
            RExpr::Seq { stmt, cnt } => {
                writeln!(f, "{stmt};")?;
                write!(f, "{cnt}")
            }
            RExpr::Assign { sym, bnd } => {
                write!(f, "{sym} = {bnd}")
            }
            RExpr::Struct { sym, fields } => {
                writeln!(f, "{sym} {{")?;
                for (field_sym, field_bnd) in fields {
                    writeln!(indented(f), "{field_sym}: {field_bnd},")?;
                }
                write!(f, "}}")
            }
            RExpr::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            RExpr::Asm { instrs } => {
                writeln!(f, "asm {{")?;
                for instr in instrs {
                    writeln!(indented(f), "{instr}")?;
                }
                write!(f, "}}")
            }
        }
    }
}
