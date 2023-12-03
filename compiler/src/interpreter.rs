use crate::passes::validate::TLit;
use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use std::collections::HashMap;
use zerocopy::AsBytes;

use std::vec::IntoIter;

pub trait IO {
    fn read(&mut self) -> Option<u8>;
    fn print(&mut self, v: u8);
}

pub struct TestIO {
    inputs: IntoIter<TLit>,
    outputs: Vec<TLit>,

    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
}

impl TestIO {
    pub fn new(inputs: Vec<TLit>) -> Self {
        Self {
            inputs: inputs.into_iter(),
            outputs: Vec::new(),

            read_buffer: Vec::new(),
            write_buffer: Vec::new(),
        }
    }

    pub fn outputs(&self) -> &Vec<TLit> {
        &self.outputs
    }
}

impl IO for TestIO {
    fn read(&mut self) -> Option<u8> {
        if self.read_buffer.is_empty() {
            if let Some(read) = self.inputs.next() {
                self.read_buffer = format!("{}\n", read.int()).into_bytes();
                self.read_buffer.reverse();
            } else {
                return None
            }
        }

        Some(self.read_buffer.pop().unwrap())
    }

    fn print(&mut self, val: u8) {
        match val {
            b'\n' => {
                let val = std::str::from_utf8(self.write_buffer.as_bytes())
                    .unwrap()
                    .parse()
                    .unwrap();
                self.outputs.push(TLit::I64 { val });
                self.write_buffer.clear();
            }
            val => {
                self.write_buffer.push(val);
            }
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug, Display)]
pub enum Val<'p> {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
    #[display(fmt = "fn pointer `{sym}`")]
    Function { sym: UniqueSym<'p> },
    #[display(fmt = "stdlib function `{sym}`")]
    StdlibFunction { sym: &'p str },
    #[display(fmt = "struct instance")]
    StructInstance { fields: HashMap<&'p str, Val<'p>> },
}

impl<'p> From<TLit> for Val<'p> {
    fn from(value: TLit) -> Self {
        match value {
            TLit::I64 { val } => Val::Int { val },
            TLit::U64 { val } => Val::Int { val: val as i64 },
            TLit::Bool { val } => Val::Bool { val },
            TLit::Unit => Val::Unit,
        }
    }
}

impl<'p> Val<'p> {
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

    pub fn fun(&self) -> UniqueSym<'p> {
        match self {
            Val::Function { sym } => *sym,
            _ => panic!(),
        }
    }

    pub fn strct(&self) -> &HashMap<&'p str, Val<'p>> {
        match self {
            Val::StructInstance { fields } => fields,
            _ => panic!(),
        }
    }
}
