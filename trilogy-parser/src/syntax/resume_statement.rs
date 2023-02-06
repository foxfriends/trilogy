use super::*;
use crate::Spanned;
use source_span::Span;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ResumeStatement {
    start: Token,
    pub expression: Option<Expression>,
}

impl Spanned for ResumeStatement {
    fn span(&self) -> Span {
        match &self.expression {
            None => self.start.span,
            Some(expression) => self.start.span.union(expression.span()),
        }
    }
}
