use crate::passes::validate::error::TypeError;
use crate::passes::validate::generate_constraints::GraphThingy;

pub type Assignments = ();

impl GraphThingy {
    pub fn solve(self) -> Result<Assignments, TypeError> {
        todo!()
    }
}