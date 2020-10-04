use crate::tokenizer;
use crate::Pos;
use std::io;
use std::io::Write;
use thiserror::Error;

/// Represents a parser error with the position in the token stream
/// where that error has happened
#[derive(PartialEq, Debug, Error)]
#[error("{kind}")]
pub struct Error {
    /// Position of an error in the token stream
    pub pos: Pos,
    /// Kind of error
    pub kind: ErrorKind,
}

impl Error {
    /// Creates new error at the specified position
    pub fn new(kind: ErrorKind, pos: Pos) -> Self {
        Self { pos, kind }
    }

    /// Reports this error to the writer
    pub fn report_to(
        &self,
        writer: &mut impl Write,
        tokens: &[tokenizer::Token],
    ) -> io::Result<()> {
        let mut offset = 0usize;
        let mut add_offset = |i: usize, s: &str| {
            if i < self.pos.0 {
                offset += s.chars().count();
            }
        };
        let mut token_size = 0;
        write!(writer, "|")?;
        for (i, text) in tokens.iter().map(|t| t.token_text()).enumerate() {
            add_offset(i, &text);
            if i == self.pos.0 {
                token_size = text.chars().count()
            }
            write!(writer, "{}", text)?;
            if i != tokens.len() - 1 {
                write!(writer, " ")?;
                add_offset(i, " ");
            }
        }
        writeln!(writer)?;
        write!(writer, "|")?;
        for _ in 0..offset {
            write!(writer, " ")?;
        }
        for _ in 0..token_size {
            write!(writer, "^")?;
        }
        writeln!(writer)?;
        writeln!(writer, "|")?;
        writeln!(writer, "= {}", self.kind)?;
        Ok(())
    }
}

/// Represents the error that a parser can output
#[derive(Error, Debug, PartialEq)]
pub enum ErrorKind {
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
        /// Identifier of the mismatched function
        id: String,
        /// Expected number of parameters to the function
        expected: usize,
        /// Actual number of parameters passed to the function
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
    /// 1 + * 1
    /// ----^
    /// |
    /// expression is expected
    #[error("Expected expression, found operator")]
    ExpectedExpression,

    /// Parser found a comma outside function
    #[error("Comma can only be used in functions, arity stack is empty")]
    CommaOutsideFn,

    /// Parser found empty parens that are not part of a function call
    #[error("Found empty parens that are not part of a function call")]
    EmptyParensNotFnCall,
}

impl ErrorKind {
    /// Enhances this [`ErrorKind`](ErrorKind) with position information
    pub fn with_pos(self, pos: Pos) -> Error {
        Error { pos, kind: self }
    }
}
