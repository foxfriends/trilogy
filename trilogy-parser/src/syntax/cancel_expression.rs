use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A cancel expression.
///
/// ```trilogy
/// cancel unit
/// ```
#[derive(Clone, Debug)]
pub struct CancelExpression {
    pub cancel: Token,
    pub expression: Expression,
    pub span: Span,
}

impl CancelExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let cancel = parser.expect(KwCancel).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: cancel.span.union(expression.span()),
            cancel,
            expression,
        })
    }
}

impl Spanned for CancelExpression {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(cancelexpr_unit: "cancel unit" => CancelExpression::parse => CancelExpression { .. });
    test_parse!(cancelexpr_value: "cancel true" => CancelExpression::parse => CancelExpression { .. });
    test_parse_error!(cancelexpr_empty: "cancel" => CancelExpression::parse);
}
