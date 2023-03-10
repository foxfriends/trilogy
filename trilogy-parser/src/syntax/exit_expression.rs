use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExitExpression {
    start: Token,
    pub expression: Expression,
}

impl ExitExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwExit)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { start, expression })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(exitexpr_unit: "exit unit" => ExitExpression::parse => "(ExitExpression _)");
    test_parse!(exitexpr_value: "exit true" => ExitExpression::parse => "(ExitExpression _)");
    test_parse_error!(exitexpr_empty: "exit" => ExitExpression::parse);
}
