use crate::passes::parse::PrgParsed;
use crate::passes::validate::error::TypeError;
use crate::passes::validate::PrgValidated;
use crate::utils::time::time;

impl<'p> PrgParsed<'p> {
    pub fn validate(self) -> Result<PrgValidated<'p>, TypeError> {
        time("parse");

        let program = self.uniquify()?.constrain()?;
        program.check_sized()?;
        program.resolve()
    }
}
