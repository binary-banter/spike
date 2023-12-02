use std::fmt::{Display, Formatter};
use indenter::indented;
use crate::passes::select::{FunSelected, X86Selected};
use std::fmt::Write;

impl Display for X86Selected<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (sym, fun) in &self.fns {
            writeln!(f, "fn {sym} {{")?;
            writeln!(indented(f), "{fun}")?;
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