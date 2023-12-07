use crate::passes::parse::PrgParsed;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgValidated;
use crate::{display, time};

impl<'p> PrgParsed<'p> {
    pub fn validate(self) -> Result<PrgValidated<'p>, TypeError> {
        let program = self.uniquify()?.constrain()?;
        program.check_sized()?;
        let program = program.resolve()?;

        display!(&program, Validate);
        time!("validate");

        Ok(program)
    }
}
