use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct SetLiteral {
    start: Token,
    pub elements: Vec<SetElement>,
    end: Token,
}

impl Spanned for SetLiteral {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

impl SetLiteral {
    pub(crate) fn new_empty(start: Token, end: Token) -> Self {
        Self {
            start,
            elements: vec![],
            end,
        }
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        first: SetElement,
    ) -> SyntaxResult<Self> {
        let mut elements = vec![first];
        if let Ok(end) = parser.expect(CBrackPipe) {
            return Ok(Self {
                start,
                elements,
                end,
            });
        };
        let end = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `|]` to end or `,` to continue set literal")
            })?;
            if let Ok(end) = parser.expect(CBrackPipe) {
                break end;
            };
            elements.push(SetElement::parse(parser)?);
            if let Ok(token) = parser.check(KwFor) {
                let error = SyntaxError::new(
                    token.span,
                    "only one element may precede the `for` keyword in a comprehension",
                );
                parser.error(error.clone());
                return Err(error);
            }
            if let Ok(end) = parser.expect(CBrackPipe) {
                break end;
            };
        };
        Ok(Self {
            start,
            elements,
            end,
        })
    }

    pub fn start_token(&self) -> &Token {
        &self.start
    }

    pub fn end_token(&self) -> &Token {
        &self.end
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum SetElement {
    Element(Expression),
    Spread(Token, Expression),
}

impl SetElement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let spread = parser.expect(OpDotDot).ok();
        let expression = Expression::parse_parameter_list(parser)?;
        match spread {
            None => Ok(Self::Element(expression)),
            Some(spread) => Ok(Self::Spread(spread, expression)),
        }
    }
}
