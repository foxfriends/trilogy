use super::{expression::Precedence, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An exit expression.
///
/// ```trilogy
/// exit 123
/// ```
#[derive(Clone, Debug)]
pub struct ExitExpression {
    pub exit: Token,
    pub expression: Expression,
    pub span: Span,
}

impl Spanned for ExitExpression {
    fn span(&self) -> Span {
        self.span
    }
}

impl ExitExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let exit = parser.expect(KwExit).unwrap();
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self {
            span: exit.span.union(expression.span()),
            exit,
            expression,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(exitexpr_unit: "exit unit" => ExitExpression::parse => ExitExpression { .. });
    test_parse!(exitexpr_value: "exit true" => ExitExpression::parse => ExitExpression { .. });
    test_parse_error!(exitexpr_empty: "exit" => ExitExpression::parse);
}
