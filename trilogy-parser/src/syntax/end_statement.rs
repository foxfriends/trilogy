use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct EndStatement {
    token: Token,
}

impl EndStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::KwEnd)
            .expect("Caller should have found this");
        Ok(Self { token })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(endstmt_empty: "end" => EndStatement::parse => "(EndStatement)");
    test_parse_error!(endstmt_value: "end unit" => EndStatement::parse);
}
