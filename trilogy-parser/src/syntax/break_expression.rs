use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct BreakExpression {
    start: Token,
    pub expression: Expression,
}

impl BreakExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwBreak)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { start, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(breakexpr_unit: "break unit" => BreakExpression::parse => "(BreakExpression _)");
    test_parse!(breakexpr_value: "break true" => BreakExpression::parse => "(BreakExpression _)");
    test_parse_error!(breakexpr_empty: "break" => BreakExpression::parse);
}
