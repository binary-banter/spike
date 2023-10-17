pub mod cvar;
pub mod lvar;
pub mod value;
pub mod x86var;

use crate::interpreter::value::Val;
use crate::language::lvar::Lit;
use std::fmt::Display;
use std::hash::Hash;
use std::io::stdin;
use std::vec::IntoIter;

pub trait IO {
    fn read(&mut self) -> Lit;
    fn print(&mut self, v: Lit);
}

struct StdIO {}

impl IO for StdIO {
    fn read(&mut self) -> Lit {
        print!("> ");
        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .expect("IO error or something");
        input
            .trim_end()
            .parse()
            .expect("Provided input was not a valid i64")
    }

    fn print(&mut self, v: Lit) {
        println!("{v}");
    }
}

pub struct TestIO {
    inputs: IntoIter<Lit>,
    outputs: Vec<Lit>,
}

impl TestIO {
    pub fn new(inputs: Vec<Lit>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<Lit> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> Lit {
        self.inputs
            .next()
            .expect("Test tried to read more input than were available.")
    }

    fn print(&mut self, v: Lit) {
        self.outputs.push(v);
    }
}
