use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndExpression {
    token: Token,
}

impl EndExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser.expect(KwEnd).expect("Caller should have found this");
        Ok(Self { token })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(endexpr_empty: "end" => EndExpression::parse => "(EndExpression)");
    test_parse_error!(endexpr_value: "end unit" => EndExpression::parse);
}
