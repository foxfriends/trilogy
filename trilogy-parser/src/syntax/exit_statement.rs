use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExitStatement {
    start: Token,
    pub expression: Expression,
}

impl ExitStatement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwExit)
            .expect("Caller should have found this");
        let expression = Expression::parse(parser)?;
        Ok(Self { start, expression })
    }

    pub fn exit_token(&self) -> &Token {
        &self.start
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(exitstmt_unit: "exit unit" => ExitStatement::parse => "(ExitStatement _)");
    test_parse!(exitstmt_value: "exit true" => ExitStatement::parse => "(ExitStatement _)");
    test_parse_error!(exitstmt_empty: "exit" => ExitStatement::parse);
}
