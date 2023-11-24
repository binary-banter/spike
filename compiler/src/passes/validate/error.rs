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
    // #[error("Type was mismatched.")]
    // MismatchedType {
    //     expect: Type<String>,
    //     got: Type<String>,
    //     #[label = "Expected this to be of type `{expect}`, but got `{got}`"]
    //     span: (usize, usize),
    // },

    // #[error("Types were mismatched. Expected '{t1}' and '{t2}' to be equal.")]
    // MismatchedTypes {
    //     t1: Type<String>,
    //     t2: Type<String>,
    //     #[label = "This has type `{t1}`"]
    //     span_t1: (usize, usize),
    //     #[label = "but this has type `{t2}`"]
    //     span_t2: (usize, usize),
    // },
    // #[error("There are multiple functions named `{sym}`.")]
    // DuplicateFunction { sym: String },
    // #[error("Function `{sym}` has duplicate argument names.")]
    // DuplicateArg { sym: String },
    #[error("Break outside loop.")]
    BreakOutsideLoop {
        #[label = "Found a break outside of a loop"]
        span: (usize, usize),
    },
    #[error("Continue outside loop.")]
    ContinueOutsideLoop {
        #[label = "Found a continue outside of a loop"]
        span: (usize, usize),
    },
    #[error("Tried to modify immutable variable.")]
    ModifyImmutable {
        #[label = "This variable was declared as immutable."]
        span: (usize, usize),
    },
    #[error("Tried to put type in variable.'")]
    SymbolShouldBeVariable {
        #[label = "This should be a variable."]
        span: (usize, usize),
    },
    #[error("Tried to use variable as type.'")]
    SymbolShouldBeStruct {
        #[label = "This should be a struct."]
        span: (usize, usize),
    },
    // #[error("The name `{sym}` should refer to a struct type.'")]
    // VariableShouldBeStruct { sym: String },
    #[error("Unknown struct field.")]
    UnknownStructField {
        sym: String,
        #[label = "The field `{sym}` is not present in the struct definition."]
        span: (usize, usize),
    },
    #[error("Missing struct field.")]
    VariableConstructMissingField {
        sym: String,
        #[label = "The field `{sym}` is missing in the struct."]
        struct_span: (usize, usize),
        #[label = "It was defined here."]
        def_span: (usize, usize),
    },
    #[error("Duplicate struct field.")]
    VariableConstructDuplicateField {
        sym: String,
        #[label = "The field `{sym}` was already provided earlier."]
        span: (usize, usize),
    },
    // #[error("The type `{typ}` should be a struct type.'")]
    // TypeShouldBeStruct { typ: Type<String> },
    // #[error("The type definition `{sym}` is not sized.'")]
    // UnsizedType { sym: String },
    #[error("Integer out of bounds.")]
    IntegerOutOfBounds {
        #[label = "This number does not fit in type `{typ}`"]
        span: (usize, usize),
        typ: &'static str,
    },
    #[error("Integer ambiguous.")]
    IntegerAmbiguous {
        #[label = "Could not determine the exact type of this integer"]
        span: (usize, usize),
    },
    #[error("The program doesn't have a main function.")]
    NoMain,
    #[error("Types did not match.")]
    MismatchedFnReturn {
        expect: String,
        got: String,

        //TODO would like this span to be return type if present
        #[label = "Expected this function to return: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    OperandExpect {
        expect: String,
        got: String,
        op: String,

        //TODO would like this span to be operator
        #[label = "Arguments of {op} are of type: `{expect}`"]
        span_op: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_arg: (usize, usize),
    },
    #[error("Types did not match.")]
    OperandEqual {
        lhs: String,
        rhs: String,
        op: String,

        //TODO would like this span to be operator
        #[label = "Arguments of {op} should be of equal types"]
        span_op: (usize, usize),
        #[label = "Type: `{lhs}`"]
        span_lhs: (usize, usize),
        #[label = "Type: `{rhs}`"]
        span_rhs: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedLetBinding {
        got: String,

        //TODO would like this span of type of let binding
        #[label = "Expected binding of let to have this type"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedAssignBinding {
        expect: String,
        got: String,

        #[label = "Expected binding of assign to have type: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    MismatchedStructField {
        expect: String,
        got: String,

        #[label = "Expected struct field to have type: `{expect}`"]
        span_expected: (usize, usize),
        #[label = "But got this type: `{got}`"]
        span_got: (usize, usize),
    },
    #[error("Types did not match.")]
    IfExpectBool {
        got: String,

        #[label = "Expected this condition to be `Bool`, but got: `{got}`"]
        span_got: (usize, usize),
    },

    #[error("Types did not match.")]
    IfExpectEqual {
        thn: String,
        els: String,

        #[label = "Type: `{thn}`"]
        span_thn: (usize, usize),
        #[label = "Type: `{els}`"]
        span_els: (usize, usize),
    },

    #[error("Types did not match.")]
    TypeMismatchExpectFn {
        got: String,

        #[label = "Expected function, but found '{got}'"]
        span_got: (usize, usize),
    },

    #[error("Types did not match.")]
    TypeMismatchLoop {
        expect: String,
        got: String,

        #[label = "Expected loop to return `{expect}`, but got: `{got}``"]
        span_break: (usize, usize),
    },

    #[error("Types did not match.")]
    ArgCountMismatch {
        expected: usize,
        got: usize,

        //TODO span of args
        #[label = "Function {expected} arguments, but found {got} arguments"]
        span: (usize, usize),
    },

    #[error("Types did not match.")]
    FnArgExpect {
        param: String,
        arg: String,

        #[label = "Expected this function argument to be of type `{param}`, but got `{arg}`"]
        span_arg: (usize, usize),
    },
}
