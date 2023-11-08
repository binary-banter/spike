use crate::passes::validate::TLit;
use derive_more::Display;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::vec::IntoIter;

pub trait IO {
    fn read(&mut self) -> TLit;
    fn print(&mut self, v: TLit);
}

pub struct TestIO {
    inputs: IntoIter<TLit>,
    outputs: Vec<TLit>,
}

impl TestIO {
    pub fn new(inputs: Vec<TLit>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<TLit> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> TLit {
        self.inputs
            .next()
            .expect("Test tried to read more input than were available.")
    }

    fn print(&mut self, v: TLit) {
        self.outputs.push(v);
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Display)]
pub enum Val<'p, A: Copy + Hash + Eq + Display> {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "fn pointer `{sym}`")]
    Function { sym: A },
    #[display(fmt = "struct instance")]
    StructInstance {
        fields: HashMap<&'p str, Val<'p, A>>,
    },
}

impl<'p, A: Copy + Hash + Eq + Display> From<TLit> for Val<'p, A> {
    fn from(value: TLit) -> Self {
        match value {
            TLit::Int { val } => Val::Int { val: val as i64 },
            TLit::Bool { val } => Val::Bool { val },
            TLit::Unit => Val::Unit,
        }
    }
}

impl<'p, A: Copy + Hash + Eq + Display> Val<'p, A> {
    pub fn int(&self) -> i64 {
        match self {
            Val::Int { val } => *val,
            _ => panic!(),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            Val::Bool { val } => *val,
            _ => panic!(),
        }
    }

    pub fn fun(&self) -> A {
        match self {
            Val::Function { sym } => *sym,
            _ => panic!(),
        }
    }

    pub fn strct(&self) -> &HashMap<&'p str, Val<'p, A>> {
        match self {
            Val::StructInstance { fields } => fields,
            _ => panic!(),
        }
    }
}
