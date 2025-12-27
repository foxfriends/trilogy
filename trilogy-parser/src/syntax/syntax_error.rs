use crate::Spanned;
use source_span::Span;
use std::fmt::{self, Display};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    Unknown(String),
    KwNotInExpression,
    RuleRightArrow,
    MatchStatementExpressionCase,
    TripleDot { dot: Span },
    IfExpressionRestriction,
    TaggedTemplateMissingIdentifier,
    TaggedTemplateNotIdentifier,
    DoMissingParameterList,
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
            ErrorKind::RuleRightArrow => {
                write!(f, "right arrow following rule should be a left arrow")?
            }
            ErrorKind::MatchStatementExpressionCase => {
                write!(f, "case in match statement should be a block")?
            }
            ErrorKind::TripleDot { .. } => write!(f, "triple `...` should be `..`")?,
            ErrorKind::IfExpressionRestriction => {
                write!(f, "an `if` expression must have an `else` clause")?
            }
            ErrorKind::TaggedTemplateMissingIdentifier => write!(
                f,
                "the $ operator prefixing a tagged template requires a tag identifier"
            )?,
            ErrorKind::TaggedTemplateNotIdentifier => write!(
                f,
                "the $ operator prefixing a tagged template requires a tag identifier"
            )?,
            ErrorKind::DoMissingParameterList => write!(
                f,
                "a `do` closure requires a parameter list, even if empty"
            )?,
        }

        Ok(())
    }
}

pub type SyntaxResult<T> = Result<T, SyntaxError>;
