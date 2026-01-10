use super::*;
use crate::{Parser, Spanned, token_pattern::TokenPattern};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

#[derive(Clone, Debug)]
pub struct FunctionHead {
    pub func: Token,
    pub name: Identifier,
    pub parameters: Vec<Pattern>,
    pub guard: Option<Guard>,
    pub span: Span,
}

impl Spanned for FunctionHead {
    fn span(&self) -> Span {
        self.span
    }
}

impl FunctionHead {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let func = parser.expect(TokenType::KwFunc).unwrap();
        let name = Identifier::parse(parser)?;
        let mut parameters = vec![];
        loop {
            parameters.push(Pattern::parse(parser)?);
            if !Pattern::PREFIX.matches(parser.peek()) {
                break;
            }
        }
        let guard = Guard::parse_optional(parser)?;
        Ok(Self {
            span: func.span.union(parameters.last().unwrap().span()),
            func,
            name,
            guard,
            parameters,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(funchead_one_param: "func hello x" => FunctionHead::parse => FunctionHead { parameters: [_], .. });
    test_parse!(funchead_multi_param: "func hello x y z" => FunctionHead::parse => FunctionHead { parameters: [_, _, _], .. });
    test_parse!(funchead_pattern_param: "func find f x:xs" => FunctionHead::parse => FunctionHead { parameters: [_, _], .. });
    test_parse!(funchead_guarded: "func find f x if x" => FunctionHead::parse => FunctionHead { parameters: [_, _], guard: Some(..), .. });
    test_parse_error!(funchead_invalid_param: "func unadd (x + y)" => FunctionHead::parse);
}
