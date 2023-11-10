use std::fmt::{Display, Formatter};
use indenter::indented;
use itertools::Itertools;
use crate::passes::parse::{Def, Expr, Op};
use std::fmt::Write;

impl<IdentVars: Display, IdentFields: Display, Expr: Display> Display for Def<IdentVars, IdentFields, Expr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Def::Fn { sym, params, typ, bdy } => { 
                writeln!(f, "fn {sym}({}) -> {typ} {{", params.iter().format(", "))?;
                writeln!(indented(f), "{bdy}")?;
                writeln!(f, "}}")?;
                Ok(())
            }
            Def::TypeDef { sym, def } => todo!(),
        }
    }
}

impl<IdentVars: Display, IdentFields: Display> Display for Expr<'_, IdentVars, IdentFields> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Lit { val } => write!(f, "{val}"),
            Expr::Var { sym } => write!(f, "{sym}"),
            Expr::Prim { op: Op::Read, ..} => write!(f, "read()"),
            Expr::Prim { op: Op::Print, args} => write!(f, "print({})", args[0].inner),
            Expr::Prim { op, args } if args.len() == 1 => write!(f, "({op} {})", args[0].inner),
            Expr::Prim { op, args } if args.len() == 2 => write!(f, "({} {op} {})", args[0].inner, args[1].inner),
            Expr::Prim { .. }  => unreachable!(),
            Expr::Let { sym, mutable, bnd, bdy } => {
                writeln!(f, "let {}{sym} = {bnd};", if *mutable { "mut " } else { "" })?;
                write!(f, "{bdy}")
            },
            Expr::If { cnd, thn, els } => {
                writeln!(f, "if {cnd} {{")?;
                writeln!(indented(f), "{thn}")?;
                writeln!(f, "}} else {{")?;
                writeln!(indented(f), "{els}")?;
                write!(f, "}}")
            }
            Expr::Apply { .. } => todo!(),
            Expr::Loop { .. } => todo!(),
            Expr::Break { .. } => todo!(),
            Expr::Continue => todo!(),
            Expr::Return { .. } => todo!(),
            Expr::Seq { .. } => todo!(),
            Expr::Assign { .. } => todo!(),
            Expr::Struct { .. } => todo!(),
            Expr::Variant { .. } => todo!(),
            Expr::AccessField { .. } => todo!(),
            Expr::Switch { .. } => todo!(),
        }
    }
}