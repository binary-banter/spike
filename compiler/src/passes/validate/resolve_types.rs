use crate::passes::validate::PrgValidated;
use crate::passes::validate::solve_constraints::Assignments;
use crate::passes::validate::uniquify::PrgUniquified;

impl<'p> PrgUniquified<'p> {
    #[must_use]
    pub fn resolve_types(self, asgns: Assignments) -> PrgValidated<'p> {
        todo!()
    }
}