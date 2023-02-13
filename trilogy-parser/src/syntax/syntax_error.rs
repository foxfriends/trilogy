use crate::Spanned;
use source_span::Span;

/// Not a real AST node, but a stand-in when a section of the code fails
/// to parse. Nodes which support recovery provide a case to hold syntax
/// errors.
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct SyntaxError {
    span: Span,
    #[allow(dead_code)]
    // for now this is used by Debug
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
}

pub type SyntaxResult<T> = Result<T, SyntaxError>;
