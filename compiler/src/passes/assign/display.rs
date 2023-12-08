use std::fmt::{Display, Formatter};
use indenter::indented;
use crate::passes::assign::{FunAssigned, X86Assigned};
use std::fmt::Write;

impl Display for X86Assigned<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, fun) in &self.fns {
            writeln!(f, "fn {sym} {{")?;
            write!(indented(f), "{fun}")?;
            writeln!(f, "}}")?;
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for FunAssigned<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, block) in &self.blocks {
            writeln!(f, "{sym}:")?;
            writeln!(f, "{block}")?;
        }

        Ok(())
    }
}
