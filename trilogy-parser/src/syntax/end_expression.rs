use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

/// An end expression.
///
/// ```trilogy
/// end
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndExpression {
    pub end: Token,
}

impl EndExpression {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let end = parser.expect(KwEnd).expect("Caller should have found this");
        Ok(Self { end })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(endexpr_empty: "end" => EndExpression::parse => "(EndExpression _)");
    test_parse_error!(endexpr_value: "end unit" => EndExpression::parse);
}
