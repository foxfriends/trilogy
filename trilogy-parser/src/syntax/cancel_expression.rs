use super::{expression::Precedence, *};
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CancelExpression {
    start: Token,
    pub expression: Expression,
}

impl CancelExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(KwCancel)
            .expect("Caller should have found this");
        let expression = Expression::parse_precedence(parser, Precedence::Continuation)?;
        Ok(Self { start, expression })
    }

    pub fn cancel_token(&self) -> &Token {
        &self.start
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(cancelexpr_unit: "cancel unit" => CancelExpression::parse => "(CancelExpression _)");
    test_parse!(cancelexpr_value: "cancel true" => CancelExpression::parse => "(CancelExpression _)");
    test_parse_error!(cancelexpr_empty: "cancel" => CancelExpression::parse);
}
