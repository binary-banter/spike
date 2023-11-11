use crate::passes::parse::{Def, Expr, Op, TypeDef};
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

impl<IdentVars: Display, IdentFields: Display> Display for Expr<'_, IdentVars, IdentFields> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit { val } => {
                write!(f, "{val}")
            }
            Expr::Var { sym } => {
                write!(f, "{sym}")
            }
            Expr::Prim { op: Op::Read, .. } => {
                write!(f, "read()")
            }
            Expr::Prim {
                op: Op::Print,
                args,
            } => {
                write!(f, "print({})", args[0].inner)
            }
            Expr::Prim { op, args } if args.len() == 1 => {
                write!(f, "({op} {})", args[0].inner)
            }
            Expr::Prim { op, args } if args.len() == 2 => {
                write!(f, "({} {op} {})", args[0].inner, args[1].inner)
            }
            Expr::Prim { .. } => {
                unreachable!()
            }
            Expr::Let {
                sym,
                mutable,
                bnd,
                bdy,
            } => {
                writeln!(
                    f,
                    "let {}{sym} = {bnd};",
                    if *mutable { "mut " } else { "" }
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
        }
    }
}
