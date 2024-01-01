use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

/// An end statement.
///
/// ```trilogy
/// end
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndStatement {
    pub end: Token,
}

impl EndStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let end = parser
            .expect(TokenType::KwEnd)
            .expect("Caller should have found this");
        Ok(Self { end })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(endstmt_empty: "end" => EndStatement::parse => "(EndStatement _)");
    test_parse_error!(endstmt_value: "end unit" => EndStatement::parse);
}
