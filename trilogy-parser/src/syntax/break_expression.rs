use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A break expression.
///
/// ```trilogy
/// break unit
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct BreakExpression {
    pub r#break: Token,
    pub expression: Expression,
    span: Span,
}

impl BreakExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#break = parser.expect(KwBreak).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#break.span.union(expression.span()),
            r#break,
            expression,
        })
    }
}

impl Spanned for BreakExpression {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(breakexpr_unit: "break unit" => BreakExpression::parse => "(BreakExpression _ _)");
    test_parse!(breakexpr_value: "break true" => BreakExpression::parse => "(BreakExpression _ _)");
    test_parse_error!(breakexpr_empty: "break" => BreakExpression::parse);
}
