use crate::debug::{Pass, DEBUG_ARGS};
use std::fmt::Display;

pub fn display<T: Display>(value: &T, pass: Pass) {
    if let Some(display) = &DEBUG_ARGS.get().and_then(|args| args.display) {
        if display == &pass {
            print!("{value}")
        }
    }
}
