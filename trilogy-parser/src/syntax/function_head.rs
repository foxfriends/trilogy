use super::*;
use crate::{token_pattern::TokenPattern, Parser};
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct FunctionHead {
    start: Token,
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
}

impl FunctionHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(TokenType::KwFunc)
            .expect("Caller should find `func` keyword.");
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        loop {
            parameters.push(Pattern::parse(parser)?);
            if Pattern::PREFIX.matches(parser.peek()) {
                continue;
            }
            return Ok(Self {
                start,
                name,
                parameters,
            });
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(funchead_one_param: "func hello x" => FunctionHead::parse => "(FunctionHead _ [_])");
    test_parse!(funchead_multi_param: "func hello x y z" => FunctionHead::parse => "(FunctionHead _ [_ _ _])");
    test_parse!(funchead_pattern_param: "func find f x:xs" => FunctionHead::parse => "(FunctionHead _ [_ _])");
    test_parse_error!(funchead_invalid_param: "func unadd (x + y)" => FunctionHead::parse);
}
