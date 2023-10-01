use crate::parser::Expression;
use crate::LVarProgram;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn uniquify_program(program: &mut LVarProgram) {
    uniquify_expression(&mut program.body, &mut HashMap::new());
}

fn uniquify_expression(expression: &mut Expression, scope: &mut HashMap<String, String>) {
    let mut stack = vec![expression];

    while let Some(expression) = stack.pop() {
        match expression {
            Expression::Int(_) => {}
            Expression::Var { name } => *name = scope[name].clone(),
            Expression::Prim { arguments, .. } => stack.extend(arguments),
            Expression::Let {
                name,
                binding,
                body,
            } => {
                uniquify_expression(binding, scope);
                let unique_name = gen_sym(name);
                scope.insert(name.clone(), unique_name.clone());
                uniquify_expression(body, scope);
                scope.remove(name);
                *name = unique_name;
            }
        }
    }
}

fn gen_sym(input: &str) -> String {
    format!("{input}_{}", COUNT.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_program;
    use crate::uniquify::uniquify_program;

    #[test]
    fn simple() {
        let mut program = parse_program("(let (x 1) 1)").unwrap().1;
        uniquify_program(&mut program);
        dbg!(program);
    }

    #[test]
    fn double_let_with_shadowing() {
        let mut program = parse_program("(let (x 1) (let (x x) 1))").unwrap().1;
        uniquify_program(&mut program);
        dbg!(program);
    }
}
