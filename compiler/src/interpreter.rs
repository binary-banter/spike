use crate::passes::parse::Lit;
use derive_more::Display;
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

#[derive(Eq, PartialEq, Copy, Clone, Debug, Display)]
pub enum Val<A: Copy + Hash + Eq + Display> {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "fn pointer `{sym}`")]
    Function { sym: A },
}

impl<A: Copy + Hash + Eq + Display> From<Lit> for Val<A> {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => Val::Int { val },
            Lit::Bool { val } => Val::Bool { val },
            Lit::Unit => Val::Unit,
        }
    }
}

impl<A: Copy + Hash + Eq + Display> Val<A> {
    pub fn int(self) -> i64 {
        match self {
            Val::Int { val } => val,
            Val::Bool { .. } => panic!(),
            Val::Function { .. } => panic!(),
            Val::Unit => panic!(),
        }
    }

    pub fn bool(self) -> bool {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { val } => val,
            Val::Function { .. } => panic!(),
            Val::Unit => panic!(),
        }
    }

    pub fn fun(self) -> A {
        match self {
            Val::Int { .. } => panic!(),
            Val::Bool { .. } => panic!(),
            Val::Function { sym } => sym,
            Val::Unit => panic!(),
        }
    }
}