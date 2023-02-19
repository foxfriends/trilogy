use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CharacterLiteral {
    token: Token,
}

impl CharacterLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Character)
            .map_err(|token| parser.expected(token, "expected character literal"))?;
        Ok(Self { token })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(char_lit: "'h'" => CharacterLiteral::parse => "(CharacterLiteral)");
    test_parse_error!(not_char_lit: "\"h\"" => CharacterLiteral::parse => "expected character literal");
}
