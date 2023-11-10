//! This module contains important definitions, structures, and parsing utilities used across different passes of the compiler.
//! It includes the grammar specification in `grammar.lalrpop` and the parsing pass.

#[rustfmt::skip]
#[allow(clippy::all, clippy::pedantic)]
mod grammar;
pub mod interpreter;
pub mod parse;
#[cfg(test)]
mod tests;
pub mod types;

use crate::utils::gen_sym::UniqueSym;
use derive_more::Display;
use functor_derive::Functor;
use std::fmt::{Display, Formatter};
use types::Type;

/// A parsed program with global definitions and an entry point.
pub struct PrgParsed<'p> {
    /// The global program definitions.
    pub defs: Vec<DefParsed<'p>>,
    /// The symbol representing the entry point of the program.
    pub entry: &'p str,
}

/// A definition.
pub enum Def<IdentVars, IdentFields, Expr> {
    /// A function definition.
    Fn {
        /// Symbol representing the function.
        sym: IdentVars,
        /// Parameters of the function.
        params: Vec<Param<IdentVars>>,
        /// Return type of the function.
        typ: Type<IdentVars>,
        /// Function body.
        bdy: Expr,
    },
    TypeDef {
        sym: IdentVars,
        def: TypeDef<IdentVars, IdentFields>,
    },
}

pub type DefParsed<'p> = Def<Spanned<&'p str>, Spanned<&'p str>, Spanned<ExprParsed<'p>>>;
pub type ExprParsed<'p> = Expr<'p, Spanned<&'p str>, Spanned<&'p str>>;

pub type DefUniquified<'p> =
    Def<Spanned<UniqueSym<'p>>, Spanned<&'p str>, Spanned<ExprUniquified<'p>>>;
pub type ExprUniquified<'p> = Expr<'p, Spanned<UniqueSym<'p>>, Spanned<&'p str>>;

pub enum TypeDef<IdentVars, IdentFields> {
    /// A struct definition.
    Struct {
        /// Fields of the struct, consisting of field symbols and their types.
        fields: Vec<(IdentFields, Type<IdentVars>)>,
    },
    /// An enum definition.
    Enum {
        /// Variants of the enum, consisting of variant symbols and their types.
        variants: Vec<(IdentFields, Type<IdentVars>)>,
    },
}

impl<IdentVars, IdentFields, Expr> Def<IdentVars, IdentFields, Expr> {
    /// Returns the symbol representing the definition.
    pub fn sym(&self) -> &IdentVars {
        match self {
            Def::Fn { sym, .. } => sym,
            Def::TypeDef { sym, .. } => sym,
        }
    }
}

/// A parameter used in functions.
///
/// Parameters are generic and can use symbols that are either `&str` or
/// [`UniqueSym`](crate::utils::gen_sym::UniqueSym) for all passes after uniquify.
#[derive(Clone)]
pub struct Param<A> {
    /// Symbol representing the parameter.
    pub sym: A,
    /// The type of the parameter. See [`Type`]
    pub typ: Type<A>,
    /// Indicates whether the parameter is mutable (true) or immutable (false).
    pub mutable: bool,
}

/// An expression.
///
/// Expressions are generic and can use symbols that are either `&str` or
/// [`UniqueSym`](crate::utils::gen_sym::UniqueSym) for all passes after uniquify.
pub enum Expr<'p, IdentVars, IdentFields> {
    /// A literal value. See [`Lit`].
    Lit {
        /// Value of the literal. See [`Lit`].
        val: Lit<'p>,
    },
    /// A variable.
    Var {
        /// Symbol representing the variable.
        sym: IdentVars,
    },
    /// A primitive operation with an arbitrary number of arguments.
    Prim {
        /// Primitive operation (e.g. `Xor`). See [`Op`].
        op: Op,
        /// Arguments used by the primitive operation.
        args: Vec<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A let binding.
    ///
    /// The `Let` expression introduces a new variable with the symbol `sym` to which it binds
    /// the `bnd` expression. It then evaluates the `bdy` expression using this new binding.
    /// The variable can be immutable or mutable depending on the presence of the `mut` keyword.
    Let {
        /// Symbol representing the newly introduced variable.
        sym: IdentVars,
        /// Indicates whether the variable is mutable (true) or immutable (false).
        mutable: bool,
        /// The expression to which the variable is bound.
        bnd: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        /// The expression that is evaluated using the new variable binding.
        bdy: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// An if statement.
    ///
    /// The `If` expression allows conditional branching. It evaluates the `cnd` expression, and if
    /// the result is true, it executes the `thn` expression; otherwise, it executes the `els` expression.
    If {
        /// The conditional expression that determines the execution path.
        cnd: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        /// The expression to execute if the condition is true.
        thn: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        /// The expression to execute if the condition is false.
        els: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A function application.
    ///
    /// The `Apply` expression signifies the invocation of a function. The `fun` expression is
    /// evaluated to obtain a function symbol, which is invoked with the arguments in `args`.
    Apply {
        /// The expression that, when evaluated, represents the function symbol to be invoked.
        fun: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        /// The ordered arguments that are passed to the function.
        args: Vec<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A loop construct.
    ///
    /// The `Loop` expression repeatedly evaluates the `bdy` expression until a `break` or `return`
    /// expression is evaluated.
    Loop {
        /// The expression that defines the body of the loop.
        bdy: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A break statement.
    ///
    /// The `Break` expression affects the control flow of a loop construct. It exits the
    /// current loop and returns the value of the `bdy` expression from the loop upon termination.
    Break {
        /// The expression to be evaluated and returned from the loop.
        bdy: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
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
        bdy: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A sequence of two expressions.
    ///
    /// The `Seq` expression combines two expressions, `stmt` and `cnt`, to be executed sequentially.
    /// The `stmt` expression is evaluated for its effects an its result discarded. Subsequently,
    /// the `cnt` expression is evaluated.
    Seq {
        /// The first expression to be executed in the sequence.
        stmt: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        /// The second expression to be executed in the sequence.
        cnt: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A variable assignment.
    ///
    /// The `Assign` expression is used to assign a new value to a variable identified by the `sym`
    /// symbol. It sets the value of the variable to the result of evaluating the `bnd` expression.
    /// Only mutable or uninitialized immutable variables can be assigned a new value.
    Assign {
        /// Symbol representing the variable to which the assignment is made.
        sym: IdentVars,
        /// The expression whose result is assigned to the variable.
        bnd: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// An instance of a struct.
    ///
    /// todo: documentation
    Struct {
        sym: IdentVars,
        fields: Vec<(IdentFields, Spanned<Expr<'p, IdentVars, IdentFields>>)>,
    },
    /// A variant of an enum.
    ///
    /// todo: documentation
    Variant {
        enum_sym: IdentVars,
        variant_sym: IdentFields,
        bdy: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
    },
    /// A field access.
    ///
    /// todo: documentation
    AccessField {
        strct: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        field: IdentFields,
    },
    /// A switch statement.
    ///
    /// todo: documentation
    Switch {
        enm: Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
        arms: Vec<SwitchArm<'p, IdentVars, IdentFields>>,
    },
}

pub type SwitchArm<'p, IdentVars, IdentFields> = (
    IdentVars,
    IdentFields,
    Box<Spanned<Expr<'p, IdentVars, IdentFields>>>,
);

#[derive(Functor, Clone)]
pub struct Spanned<T> {
    pub span: (usize, usize),
    pub inner: T,
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.inner)
    }
}

/// A primitive operation.
pub enum Op {
    /// Read signed integer from stdin.
    Read,
    /// Print signed integer to stdout.
    Print,
    /// Integer addition.
    Plus,
    /// Integer subtraction or negation.
    Minus,
    /// Integer multiplication.
    Mul,
    /// Integer division.
    Div,
    /// Modulo operation.
    Mod,
    /// Logical AND.
    LAnd,
    /// Logical OR,
    LOr,
    /// Logical NOT.
    Not,
    /// XOR operation.
    Xor,
    /// Greater Than comparison.
    GT,
    /// Greater Than or Equal To comparison.
    GE,
    /// Equality comparison. Operates on `Int` and `Bool`.
    EQ,
    /// Less Than or Equal To comparison.
    LE,
    /// Less Than comparison.
    LT,
    /// Inequality comparison. Operates on `Int` and `Bool`.
    NE,
}

/// A literal value.
#[derive(Display)]
pub enum Lit<'p> {
    /// Integer literal, representing a signed 64-bit number.
    #[display(fmt = "{val}")]
    Int { val: &'p str },
    /// Boolean literal, representing a value of *true* or *false*.
    #[display(fmt = "{}", r#"if *val { "true" } else { "false" }"#)]
    Bool { val: bool },
    /// Unit literal, representing the absence of a value.
    #[display(fmt = "unit")]
    Unit,
}
