use crate::lvar::{Expr, LVarProgram, Type};
use crate::utils::push_map::PushMap;

#[derive(Debug)]
pub enum TypeError {
    UndeclaredVar,
}

pub fn type_check_program(program: &LVarProgram) -> Result<(), TypeError> {
    type_check_expr(&program.bdy, &mut PushMap::default())
}

fn type_check_expr(expr: &Expr, scope: &mut PushMap<String, Type>) -> Result<(), TypeError> {
    match expr {
        Expr::Int { .. } => Ok(()),
        Expr::Var { sym } => if scope.contains(sym) {Ok(())} else {Err(TypeError::UndeclaredVar)},
        Expr::Prim { args, .. } => {
            for arg in args{
                type_check_expr(arg, scope)?;
            }
            Ok(())
        },
        Expr::Let { sym, bnd, bdy } => {
            type_check_expr(bnd, scope)?;
            scope.push(sym.clone(), Type::Integer, |scope| {
                type_check_expr(bdy, scope)
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_program;
    use test_each_file::test_each_file;
    use crate::type_checking::type_check_program;

    fn check([test]: [&str; 1], should_fail: bool) {
        let mut test = test.split("#");
        let program = test.nth(3).unwrap().trim();
        let program = parse_program(program).unwrap().1;

        if should_fail {
            assert!(type_check_program(&program).is_err());
        } else{
            assert!(type_check_program(&program).is_ok());
        }
    }


    test_each_file! { for ["test"] in "./programs/good" as good => |p| check(p, false) }
    test_each_file! { for ["test"] in "./programs/type_fail" as bad => |p| check(p, true) }
}