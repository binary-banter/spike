use crate::passes::parse::types::Type;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum TypeError {
    #[error("Encountered an undeclared variable.")]
    UndeclaredVar {
        sym: String,
        #[label = "This variable `{sym}` was not declared yet"]
        span: (usize, usize),
    },
    #[error("Type was mismatched.")]
    MismatchedType {
        expect: Type<String>,
        got: Type<String>,
        #[label = "Expected this to be of type `{expect}`, but got `{got}`"]
        span: (usize, usize),
    },
    #[error("Types were mismatched. Expected function, but found '{got}'.")]
    TypeMismatchExpectFn { got: Type<String> },
    #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    MismatchedTypes {
        t1: Type<String>,
        t2: Type<String>,
        #[label = "This has type `{t1}`"]
        span_t1: (usize, usize),
        #[label = "but this has type `{t2}`"]
        span_t2: (usize, usize),
    },
    #[error("There are multiple functions named `{sym}`.")]
    DuplicateFunction { sym: String },
    #[error("Function `{sym}` has duplicate argument names.")]
    DuplicateArg { sym: String },
    #[error("Function `{expected}` has {expected} arguments, but found {got} arguments.")]
    ArgCountMismatch { expected: usize, got: usize },
    #[error("Found a break outside of a loop.")]
    BreakOutsideLoop,
    #[error("Tried to modify immutable variable '{sym}'")]
    ModifyImmutable { sym: String },
    #[error("The name {sym} should refer to a variable binding.'")]
    VariableShouldBeExpr { sym: String },
    #[error("The name `{sym}` should refer to a struct type.'")]
    VariableShouldBeStruct { sym: String },
    #[error("The field `{sym}` is not present in the struct definition.'")]
    UnknownStructField { sym: String },
    #[error("The field `{sym}` is missing in the struct.'")]
    VariableConstructMissingField { sym: String },
    #[error("The field `{sym}` was already provided earlier.'")]
    VariableConstructDuplicateField { sym: String },
    #[error("The type `{typ}` should be a struct type.'")]
    TypeShouldBeStruct { typ: Type<String> },
    #[error("The type definition `{sym}` is not sized.'")]
    UnsizedType { sym: String },

    #[error("Integer out of bounds.")]
    IntegerOutOfBounds {
        #[label = "This number does not fit in an i32: `-2147483648..=2147483647`"]
        span: (usize, usize),
    },
    #[error("The program doesn't have a main function.")]
    NoMain,
}