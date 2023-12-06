use crate::passes::parse::{Def, Expr, TypeDef};
use indenter::indented;
use itertools::Itertools;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl<IdentVars: Display, IdentFields: Display, Expr: Display> Display
    for Def<IdentVars, IdentFields, Expr>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Def::Fn {
                sym,
                params,
                typ,
                bdy,
            } => {
                writeln!(f, "fn {sym}({}) -> {typ} {{", params.iter().format(", "))?;
                writeln!(indented(f), "{bdy}")?;
                writeln!(f, "}}")?;
                Ok(())
            }
            Def::TypeDef { sym, def } => match def {
                TypeDef::Struct { fields } => {
                    writeln!(f, "struct {sym} {{")?;
                    for (field_sym, field_typ) in fields {
                        writeln!(indented(f), "{field_sym}: {field_typ},")?;
                    }
                    writeln!(f, "}}")
                }
                TypeDef::Enum { .. } => todo!(),
            },
        }
    }
}

impl<IdentVars: Display, IdentFields: Display, Lit: Display, M> Display
    for Expr<IdentVars, IdentFields, Lit, M>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit { val } => {
                write!(f, "{val}")
            }
            Expr::Var { sym } => {
                write!(f, "{sym}")
            }
            Expr::UnaryOp { op, expr } => {
                write!(f, "({op}{expr})")
            }
            Expr::BinaryOp {
                op,
                exprs: [e1, e2],
            } => {
                write!(f, "({e1} {op} {e2})")
            }
            Expr::Let {
                sym,
                mutable,
                typ,
                bnd,
                bdy,
            } => {
                let mutable = if *mutable { "mut " } else { "" };
                let typ = typ
                    .as_ref()
                    .map(|typ| format!(": {typ}"))
                    .unwrap_or_default();

                writeln!(f, "let {mutable}{sym}{typ} = {bnd};")?;
                write!(f, "{bdy}")
            }
            Expr::If { cnd, thn, els } => {
                writeln!(f, "if {cnd} {{")?;
                writeln!(indented(f), "{thn}")?;
                writeln!(f, "}} else {{")?;
                writeln!(indented(f), "{els}")?;
                write!(f, "}}")
            }
            Expr::Apply { fun, args } => {
                write!(f, "{fun}({})", args.iter().format(", "))
            }
            Expr::Loop { bdy } => {
                writeln!(f, "loop {{")?;
                writeln!(indented(f), "{bdy}")?;
                write!(f, "}}")
            }
            Expr::Break { bdy } => {
                write!(f, "break {bdy}")
            }
            Expr::Continue => {
                write!(f, "continue")
            }
            Expr::Return { bdy } => {
                write!(f, "return {bdy}")
            }
            Expr::Seq { stmt, cnt } => {
                writeln!(f, "{stmt};")?;
                write!(f, "{cnt}")
            }
            Expr::Assign { sym, bnd } => {
                write!(f, "{sym} = {bnd}")
            }
            Expr::Struct { sym, fields } => {
                writeln!(f, "{sym} {{")?;
                for (field_sym, field_bnd) in fields {
                    writeln!(indented(f), "{field_sym}: {field_bnd},")?;
                }
                write!(f, "}}")
            }
            Expr::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            Expr::Asm { instrs } => {
                writeln!(f, "asm {{")?;
                for instr in instrs {
                    writeln!(indented(f), "{instr}")?;
                }
                write!(f, "}}")
            }
            Expr::Variant { .. } => todo!(),
            Expr::Switch { .. } => todo!(),
        }
    }
}
