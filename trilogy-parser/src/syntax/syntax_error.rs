use crate::Spanned;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct SyntaxError {
    span: Span,
    message: String,
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
            message: message.to_string(),
        }
    }

    pub(crate) fn new_spanless(message: impl std::fmt::Display) -> Self {
        Self {
            span: Span::default(),
            message: message.to_string(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

pub type SyntaxResult<T> = Result<T, SyntaxError>;
