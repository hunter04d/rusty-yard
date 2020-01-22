use thiserror::Error;

/// Represents the error that a parser can output
#[derive(Error, Debug, PartialEq)]
#[error("Parser Error")]
pub enum Error {
    /// left paren has not been found after identifier that represents a function
    #[error("Expected left paren after function id")]
    NoLeftParenAfterFnId,
    /// Bad token found in input
    #[error("Bad token {0:?}")]
    BadToken(String),

    /// Operator at the end of the expression
    #[error("Operator at the end of the token stream")]
    OperatorAtTheEnd,

    /// Mismatched left parenthesis
    #[error("Mismatched left paren in the token stream")]
    MismatchedLeftParen,

    /// Mismatched right parenthesis
    #[error("Mismatched right paren in the token stream")]
    MismatchedRightParen,

    /// Signifies that a function has been called with different number of parameters than expected
    #[error("Arity of function {id} mismatched: expected: {expected}, actual: {actual}")]
    ArityMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },

    /// Expected an operator in the input but found expression
    ///
    /// # Example
    ///
    /// 1 10 + 10
    /// --^^
    /// |
    /// operator is expected
    #[error("Expected Operator, found expression")]
    ExpectedOperator,

    /// Expected an expression in the input but found operator
    ///
    /// # Example
    ///
    /// 1 + + 1
    /// ----^
    /// |
    /// expression is expected
    #[error("Expected expression, found operator")]
    ExpectedExpression,

    /// Parser found a comma outside function
    #[error("Comma can only be used in functions, arity stack is empty")]
    CommaOutsideFn,
}
