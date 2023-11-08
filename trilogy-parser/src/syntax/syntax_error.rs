use crate::Spanned;
use source_span::Span;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    Unknown(String),
    KwNotInExpression,
}

impl ErrorKind {
    pub(crate) fn at(self, span: Span) -> SyntaxError {
        SyntaxError { span, kind: self }
    }
}

#[derive(Clone, Debug)]
pub struct SyntaxError {
    span: Span,
    kind: ErrorKind,
}

impl Spanned for SyntaxError {
    fn span(&self) -> Span {
        self.span
    }
}

impl SyntaxError {
    pub(crate) fn new(span: Span, message: impl std::fmt::Display) -> Self {
        Self {
            span,
            kind: ErrorKind::Unknown(message.to_string()),
        }
    }

    pub(crate) fn new_spanless(message: impl std::fmt::Display) -> Self {
        Self {
            span: Span::default(),
            kind: ErrorKind::Unknown(message.to_string()),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl std::error::Error for SyntaxError {}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "syntax error ({}): ", self.span)?;
        match &self.kind {
            ErrorKind::Unknown(message) => write!(f, "{message}")?,
            ErrorKind::KwNotInExpression => {
                write!(f, "keyword `not` cannot be used in an expression")?
            }
        }

        Ok(())
    }
}

pub type SyntaxResult<T> = Result<T, SyntaxError>;
