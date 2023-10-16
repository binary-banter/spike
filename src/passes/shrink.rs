//   def shrinkProgram(p: Lfun): Lfun = p match {
//     case Lfun.ProgramDefsExp(defs, e) => Lfun.ProgramDefs(mainDef(e) :: defs.map(shrinkDef))
//   }
//
//   def mainDef(body: Expr) : Def = {
//     FunctionDef("main", Nil, IntT(), shrinkExpr(body))
//   }
//
//   def shrinkDef(definition: Def) : Def = definition match {
//     case FunctionDef(name, args, rtrn, body) => FunctionDef(name, args, rtrn, shrinkExpr(body))
//   }
//
//   def shrinkExpr(expression: Expr) : Expr = {
//     expression match {
//       case Prim(And(), List(e1, e2)) => If(shrinkExpr(e1), shrinkExpr(e2), Bool(false))
//       case Prim(Or(), List(e1, e2)) => If(shrinkExpr(e1), Bool(true), shrinkExpr(e2))
//       case Prim(c, e) => Prim(c, e.map(shrinkExpr))
//       case Let(x, e1, e2) => Let(x, shrinkExpr(e1), shrinkExpr(e2))
//       case If(c, t, e) => If(shrinkExpr(c), shrinkExpr(t), shrinkExpr(e))
//       case _ => expression
//     }
//   }
// }

use crate::language::lvar::{Def, LVarProgram, SVarProgram};
use crate::type_checking::Type;

impl<'p> LVarProgram<'p>{
    pub fn shrink(mut self) -> SVarProgram<'p> {
        self.defs.push(Def::Fn {sym: "main", args: vec![], typ: Type::Int, bdy: self.bdy});

        SVarProgram{
            defs: self.defs,
            entry: "main",
        }
    }
}