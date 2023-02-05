use source_span::Span;

/// Not a real AST node, but a stand-in when a section of the code fails
/// to parse. Nodes which support recovery provide a case to hold syntax
/// errors.
#[derive(Clone, Debug)]
pub struct SyntaxError {
    span: Option<Span>,
    message: String,
}

impl SyntaxError {
    pub(crate) fn new(span: Span, message: impl std::fmt::Display) -> Self {
        Self {
            span: Some(span),
            message: message.to_string(),
        }
    }

    pub(crate) fn new_spanless(message: impl std::fmt::Display) -> Self {
        Self {
            span: None,
            message: message.to_string(),
        }
    }
}

pub type SyntaxResult<T> = Result<T, SyntaxError>;
