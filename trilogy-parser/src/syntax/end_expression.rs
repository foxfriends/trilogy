use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An end expression.
///
/// ```trilogy
/// end
/// ```
#[derive(Clone, Debug)]
pub struct EndExpression {
    pub end: Token,
    pub span: Span,
}

impl Spanned for EndExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl EndExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let end = parser.expect(KwEnd).unwrap();
        Ok(Self {
            span: end.span,
            end,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(endexpr_empty: "end" => EndExpression::parse => EndExpression { .. });
    test_parse_error!(endexpr_value: "end unit" => EndExpression::parse);
}
