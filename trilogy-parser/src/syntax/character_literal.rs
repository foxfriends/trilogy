use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType};

/// A character literal expression.
///
/// ```trilogy
/// 'a'
/// ```
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct CharacterLiteral {
    pub token: Token,
}

impl CharacterLiteral {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let token = parser
            .expect(TokenType::Character)
            .map_err(|token| parser.expected(token, "expected character literal"))?;
        Ok(Self { token })
    }

    pub fn value(&self) -> char {
        *self.token.value.as_ref().unwrap().as_char().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(char_lit: "'h'" => CharacterLiteral::parse => "(CharacterLiteral _)");
    test_parse_error!(not_char_lit: "\"h\"" => CharacterLiteral::parse => "expected character literal");
}
