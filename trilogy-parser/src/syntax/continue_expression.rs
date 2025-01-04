use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// A continue expression.
///
/// ```trilogy
/// continue unit
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ContinueExpression {
    pub r#continue: Token,
    pub expression: Expression,
    span: Span,
}

impl ContinueExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#continue = parser.expect(KwContinue).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: r#continue.span.union(expression.span()),
            r#continue,
            expression,
        })
    }
}

impl Spanned for ContinueExpression {
    fn span(&self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(continueexpr_unit: "continue unit" => ContinueExpression::parse => "(ContinueExpression _ _)");
    test_parse!(continueexpr_value: "continue true" => ContinueExpression::parse => "(ContinueExpression _ _)");
    test_parse_error!(continueexpr_empty: "continue" => ContinueExpression::parse);
}
