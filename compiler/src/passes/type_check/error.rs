use thiserror::Error;
use miette::Diagnostic;
use crate::passes::parse::types::Type;

#[derive(Debug, Error, Diagnostic)]
#[diagnostic()]
pub enum TypeError {
    #[error("Variable '{sym}' was not declared yet.")]
    UndeclaredVar { sym: String },
    #[error("Types were mismatched. Expected '{expect}', but found '{got}'.")]
    TypeMismatchExpect {
        expect: Type<String>,
        got: Type<String>,
    },
    #[error("Types were mismatched. Expected function, but found '{got}'.")]
    TypeMismatchExpectFn { got: Type<String> },
    #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    TypeMismatchEqual { t1: Type<String>, t2: Type<String> },
    #[error("There are multiple functions named `{sym}`.")]
    DuplicateFunction { sym: String },
    #[error("Function `{sym}` has duplicate argument names.")]
    DuplicateArg { sym: String },
    #[error("Function `{expected}` has {expected} arguments, but found {got} arguments.")]
    ArgCountMismatch { expected: usize, got: usize },
    #[error("The program doesn't have a main function.")]
    NoMain,
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
}
