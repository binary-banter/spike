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
                    writeln!(
                        indented(f),
                        "{}",
                        fields
                            .iter()
                            .map(|(sym, bnd)| format!("{sym}: {bnd},"))
                            .format("\n")
                    )?;
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
                writeln!(
                    f,
                    "let {}{sym}{} = {bnd};",
                    if *mutable { "mut " } else { "" },
                    typ.as_ref()
                        .map(|typ| format!(": {typ}"))
                        .unwrap_or("".to_string())
                )?;
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
            Expr::AccessField { strct, field } => {
                write!(f, "{strct}.{field}")
            }
            Expr::Variant { .. } => todo!(),
            Expr::Switch { .. } => todo!(),
            Expr::Asm { .. } => todo!(),
        }
    }
}
