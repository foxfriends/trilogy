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
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        first: SetElement,
    ) -> SyntaxResult<Self> {
        let mut elements = vec![first];
        let end = loop {
            if let Ok(token) = parser.check(KwFor) {
                let error = SyntaxError::new(
                    token.span,
                    "only one element may precede the `for` keyword in a comprehension",
                );
                parser.error(error.clone());
                return Err(error);
            }
            let token = parser.expect([CBracePipe, OpComma]).map_err(|token| {
                parser.expected(token, "expected `|}` to end or `,` to continue set literal")
            })?;
            if token.token_type == CBracePipe {
                break token;
            }
            elements.push(SetElement::parse(parser)?);
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
