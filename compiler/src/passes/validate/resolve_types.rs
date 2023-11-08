use crate::passes::validate::solve_constraints::Assignments;
use crate::passes::validate::uniquify::PrgUniquified;
use crate::passes::validate::PrgValidated;

impl<'p> PrgUniquified<'p> {
    #[must_use]
    pub fn resolve_types(self, _asgns: Assignments) -> PrgValidated<'p> {
        todo!()
    }
}
