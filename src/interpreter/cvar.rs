// use crate::interpreter::lvar::interpret_expr;
use crate::interpreter::value::Val;
use crate::interpreter::IO;
use crate::language::cvar::{PrgExplicated, Tail};
use crate::passes::uniquify::UniqueSym;
use crate::utils::push_map::PushMap;

impl<'p> PrgExplicated<'p> {
    pub fn interpret(&self, io: &mut impl IO) -> Val<UniqueSym<'p>> {
        todo!()
        // self.interpret_tail(&self.blocks[&self.entry], &mut PushMap::default(), io)
    }

    // fn interpret_tail(
    //     &self,
    //     tail: &Tail<'p>,
    //     scope: &mut PushMap<UniqueSym<'p>, Val>,
    //     io: &mut impl IO,
    // ) -> Val {
    //     match tail {
    //         Tail::Return { expr } => interpret_expr(&expr.clone().into(), scope, io),
    //         Tail::Seq { sym, bnd, tail } => {
    //             let bnd = interpret_expr(&bnd.clone().into(), scope, io);
    //             scope.push(*sym, bnd, |scope| self.interpret_tail(tail, scope, io))
    //         }
    //         Tail::IfStmt { cnd, thn, els } => {
    //             if interpret_expr(&cnd.clone().into(), scope, io).bool() {
    //                 self.interpret_tail(&self.blocks[thn], scope, io)
    //             } else {
    //                 self.interpret_tail(&self.blocks[els], scope, io)
    //             }
    //         }
    //         Tail::Goto { lbl } => self.interpret_tail(&self.blocks[lbl], scope, io),
    //     }
    // }
}
