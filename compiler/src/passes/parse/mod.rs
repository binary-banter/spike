#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic)]
mod grammar;
pub mod interpreter;
pub mod parse;
pub mod types;

use derive_more::Display;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::str::FromStr;
use types::Type;

#[derive(Debug, PartialEq)]
pub struct PrgParsed<'p> {
    pub defs: Vec<Def<&'p str, Expr<&'p str>>>,
    pub entry: &'p str,
}

#[derive(Debug, PartialEq)]
pub struct PrgGenericVar<A: Copy + Hash + Eq + Display> {
    pub defs: HashMap<A, Def<A, Expr<A>>>,
    pub entry: A,
}

#[derive(Debug, PartialEq)]
pub enum Def<A: Copy + Hash + Eq + Display, B> {
    Fn {
        sym: A,
        params: Vec<Param<A>>,
        typ: Type<A>,
        bdy: B,
    },
    Struct {
        sym: A,
        fields: Vec<(A, Type<A>)>,
    },
    Enum {
        sym: A,
        variants: Vec<(A, Type<A>)>,
    },
}

impl<A: Copy + Hash + Eq + Display, B> Def<A, B> {
    pub fn sym(&self) -> &A {
        match self {
            Def::Fn { sym, .. } => sym,
            Def::Struct { sym, .. } => sym,
            Def::Enum { sym, .. } => sym,
        }
    }
}

/// A parameter used in functions.
///
/// Parameters are generic and can use symbols that are either `&str` or
/// [`UniqueSym`](crate::utils::gen_sym::UniqueSym) for all passes after uniquify.
#[derive(Debug, PartialEq)]
pub struct Param<A: Copy + Hash + Eq + Display> {
    /// Symbol representing the parameter.
    pub sym: A,
    /// The type of the parameter. See [`Type`]
    pub typ: Type<A>,
    /// Indicates whether the parameter is mutable (true) or immutable (false).
    pub mutable: bool,
}

/// An expression in our custom programming language.
///
/// Expressions are generic and can use symbols that are either `&str` or
/// [`UniqueSym`](crate::utils::gen_sym::UniqueSym) for all passes after uniquify.
#[derive(Debug, PartialEq)]
pub enum Expr<A: Copy + Hash + Eq> {
    /// A literal value. See [`Lit`].
    Lit {
        /// Value of the literal. See [`Lit`].
        val: Lit,
    },
    /// A variable.
    Var {
        /// Symbol representing the variable.
        sym: A,
    },
    /// A primitive operation with an arbitrary number of arguments.
    Prim {
        /// Primitive operation (e.g. `Xor`). See [`Op`].
        op: Op,
        /// Arguments used by the primitive operation.
        args: Vec<Expr<A>>,
    },
    /// A let binding.
    ///
    /// The `Let` expression introduces a new variable with the symbol `sym` to which it binds
    /// the `bnd` expression. It then evaluates the `bdy` expression using this new binding.
    /// The variable can be immutable or mutable depending on the presence of the `mut` keyword.
    Let {
        /// Symbol representing the newly introduced variable.
        sym: A,
        /// Indicates whether the variable is mutable (true) or immutable (false).
        mutable: bool,
        /// The expression to which the variable is bound.
        bnd: Box<Expr<A>>,
        /// The expression that is evaluated using the new variable binding.
        bdy: Box<Expr<A>>,
    },
    /// An if statement.
    ///
    /// The `If` expression allows conditional branching. It evaluates the `cnd` expression, and if
    /// the result is true, it executes the `thn` expression; otherwise, it executes the `els` expression.
    If {
        /// The conditional expression that determines the execution path.
        cnd: Box<Expr<A>>,
        /// The expression to execute if the condition is true.
        thn: Box<Expr<A>>,
        /// The expression to execute if the condition is false.
        els: Box<Expr<A>>,
    },
    /// A function application.
    ///
    /// The `Apply` expression signifies the invocation of a function. The `fun` expression is
    /// evaluated to obtain a function symbol, which is invoked with the arguments in `args`.
    Apply {
        /// The expression that, when evaluated, represents the function symbol to be invoked.
        fun: Box<Expr<A>>,
        /// The ordered arguments that are passed to the function.
        args: Vec<Expr<A>>,
    },
    /// A loop construct.
    ///
    /// The `Loop` expression repeatedly evaluates the `bdy` expression until a `break` or `return`
    /// expression is evaluated.
    Loop {
        /// The expression that defines the body of the loop.
        bdy: Box<Expr<A>>,
    },
    /// A break statement.
    ///
    /// The `Break` expression affects the control flow of a loop construct. It exits the
    /// current loop and returns the value of the `bdy` expression from the loop upon termination.
    Break {
        /// The expression to be evaluated and returned from the loop.
        bdy: Box<Expr<A>>,
    },
    /// A continue statement.
    ///
    /// The `Continue` expression affects the control flow of a loop construct. It skips to
    /// the next iteration of the loop. It does not return a value.
    Continue,
    /// A return statement.
    ///
    /// The `Return` expression exits the current function and returns the value of the `bdy` expression.
    Return {
        /// The expression to be evaluated and returned from the function.
        bdy: Box<Expr<A>>,
    },
    /// A sequence of two expressions.
    ///
    /// The `Seq` expression combines two expressions, `stmt` and `cnt`, to be executed sequentially.
    /// The `stmt` expression is evaluated for its effects an its result discarded. Subsequently,
    /// the `cnt` expression is evaluated.
    Seq {
        /// The first expression to be executed in the sequence.
        stmt: Box<Expr<A>>,
        /// The second expression to be executed in the sequence.
        cnt: Box<Expr<A>>,
    },
    /// A variable assignment.
    ///
    /// The `Assign` expression is used to assign a new value to a variable identified by the `sym`
    /// symbol. It sets the value of the variable to the result of evaluating the `bnd` expression.
    /// Only mutable or uninitialized immutable variables can be assigned a new value.
    Assign {
        /// Symbol representing the variable to which the assignment is made.
        sym: A,
        /// The expression whose result is assigned to the variable.
        bnd: Box<Expr<A>>,
    },
    /// An instance of a struct.
    ///
    /// todo: documentation
    Struct { sym: A, fields: Vec<(A, Expr<A>)> },
    /// A variant of an enum.
    ///
    /// todo: documentation
    Variant {
        enum_sym: A,
        variant_sym: A,
        bdy: Box<Expr<A>>,
    },
    /// A field access.
    ///
    /// todo: documentation
    AccessField { strct: Box<Expr<A>>, field: A },
    /// A switch statement.
    ///
    /// todo: documentation
    Switch {
        enm: Box<Expr<A>>,
        arms: Vec<(A, A, Box<Expr<A>>)>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Op {
    Read,
    Print,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    LAnd,
    LOr,
    Not,
    Xor,
    GT,
    GE,
    EQ,
    LE,
    LT,
    NE,
}

#[derive(Copy, Clone, Debug, PartialEq, Display)]
pub enum Lit {
    #[display(fmt = "{val}")]
    Int { val: i64 },
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    #[display(fmt = "unit")]
    Unit,
}

impl Lit {
    #[must_use]
    pub fn int(self) -> i64 {
        if let Lit::Int { val } = self {
            val
        } else {
            panic!()
        }
    }

    #[must_use]
    pub fn bool(self) -> bool {
        if let Lit::Bool { val } = self {
            val
        } else {
            panic!()
        }
    }
}

impl From<Lit> for i64 {
    fn from(value: Lit) -> Self {
        match value {
            Lit::Int { val } => val,
            Lit::Bool { val } => val as i64,
            Lit::Unit => 0,
        }
    }
}

impl FromStr for Lit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "false" => Lit::Bool { val: false },
            "true" => Lit::Bool { val: true },
            "unit" => Lit::Unit,
            s => Lit::Int {
                val: s.parse().map_err(|_| ())?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::split_test::split_test;
    use test_each_file::test_each_file;

    fn parse([test]: [&str; 1]) {
        let _ = split_test(test);
    }

    test_each_file! { for ["test"] in "./programs/good" as parse => parse }
}
