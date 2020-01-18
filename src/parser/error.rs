use thiserror::Error;

#[derive(Error, Debug)]
#[error("Parser Error")]
pub enum Error {
    #[error("Expected left paren after function id")]
    NoLeftParenAfterFnId,
    #[error("Bad token {0:?}")]
    BadToken(String),

    #[error("Operator at the end of the token stream")]
    OperatorAtTheEnd,

    #[error("Mismatched left paren in the token stream")]
    MismatchedLeftParen,

    #[error("Mismatched right paren in the token stream")]
    MismatchedRightParen,

    #[error("Arity of function {id} mismatched: expected: {expected}, actual: {actual}")]
    ArityMismatch {
        id: String,
        expected: usize,
        actual: usize,
    },
    
    #[error("Expected Operator, found expression")]
    ExpectedOperator,

    #[error("Expected expression, found operator")]
    ExpectedExpression,

    #[error("Comma can only be used in functions, arity stack is empty")]
    CommaOutsideFn,
}
