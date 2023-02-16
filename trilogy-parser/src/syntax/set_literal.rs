use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct SetLiteral {
    start: Token,
    pub elements: Vec<SetElement>,
    end: Token,
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
        parser.expect(OpComma).map_err(|token| {
            parser.expected(token, "expected `|}` to end or `,` to continue set literal")
        })?;
        let end = loop {
            if let Ok(end) = parser.expect(CBracePipe) {
                break end;
            };
            if let Ok(token) = parser.check(KwFor) {
                let error = SyntaxError::new(
                    token.span,
                    "only one element may precede the `for` keyword in a comprehension",
                );
                parser.error(error.clone());
                return Err(error);
            }
            elements.push(SetElement::parse(parser)?);
            if let Ok(end) = parser.expect(CBracePipe) {
                break end;
            };
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `|}` to end or `,` to continue set literal")
            })?;
        };
        Ok(Self {
            start,
            elements,
            end,
        })
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
