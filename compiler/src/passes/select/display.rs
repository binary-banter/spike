use crate::passes::select::{FunSelected, X86Selected};
use indenter::indented;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

impl Display for X86Selected<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, fun) in &self.fns {
            writeln!(f, "fn {sym} {{")?;
            write!(indented(f), "{fun}")?;
            writeln!(f, "}}")?;
        }

        Ok(())
    }
}

impl Display for FunSelected<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, block) in &self.blocks {
            writeln!(f, "{sym}:")?;
            writeln!(f, "{block}")?;
        }

        Ok(())
    }
}
