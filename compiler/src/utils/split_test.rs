use crate::interpreter::Val;
use crate::passes::parse::parse::parse_program;
use crate::passes::parse::{Lit, PrgParsed};
use crate::passes::validate::TLit;
use std::cell::OnceCell;
use std::hash::Hash;
use std::str::SplitWhitespace;

/// Splits the inputs, expected outputs and expected return from the test.
/// The values must be preceded by `//*` and `inp:`, `out:` or `ret:`.
#[must_use]
pub fn split_test(test: &str) -> (Vec<TLit>, Vec<TLit>, TLit, PrgParsed) {
    let mut input = OnceCell::new();
    let mut output = OnceCell::new();
    let mut expected_return = OnceCell::new();

    for line in test.lines() {
        let mut parts = line.split_whitespace();

        match (parts.next(), parts.next()) {
            (Some("//*"), Some("inp:")) => input
                .set(parts.map(str::parse).collect::<Result<_, _>>().unwrap())
                .unwrap(),
            (Some("//*"), Some("out:")) => output
                .set(parts.map(str::parse).collect::<Result<_, _>>().unwrap())
                .unwrap(),
            (Some("//*"), Some("ret:")) => expected_return
                .set(parts.next().unwrap().parse().unwrap())
                .unwrap(),
            _ => {}
        }
    }

    (
        input.take().unwrap_or_default(),
        output.take().unwrap_or_default(),
        expected_return.take().unwrap_or(TLit::Unit),
        parse_program(test).unwrap(), // todo: pass test file name
    )
}
