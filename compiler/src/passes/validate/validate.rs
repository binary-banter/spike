use crate::passes::parse::PrgParsed;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgValidated;

impl<'p> PrgParsed<'p> {
    pub fn validate(self) -> Result<PrgValidated<'p>, TypeError> {
        let program = self.uniquify()?.constrain()?;
        todo!()
        // let program = program.resolve_types(assignments);
        // program.check_sized()?;
        // Ok(program)
    }
}
