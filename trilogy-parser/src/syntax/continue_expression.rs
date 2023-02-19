use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ContinueExpression {
    start: Token,
    pub expression: Expression,
}

impl ContinueExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwContinue)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { start, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(continueexpr_unit: "continue unit" => ContinueExpression::parse => "(ContinueExpression _)");
    test_parse!(continueexpr_value: "continue true" => ContinueExpression::parse => "(ContinueExpression _)");
    test_parse_error!(continueexpr_empty: "continue" => ContinueExpression::parse);
}
