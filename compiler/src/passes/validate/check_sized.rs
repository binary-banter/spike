use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgValidated;

impl<'p> PrgValidated<'p> {
    pub fn check_sized(&self) -> Result<(), TypeError> {
        todo!()
    }
}
