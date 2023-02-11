use super::*;
use crate::Parser;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct RecordLiteral {
    start: Token,
    pub elements: Vec<RecordElement>,
    end: Token,
}

impl RecordLiteral {
    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        first: RecordElement,
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
            let token = parser.expect([CBrace, OpComma]).map_err(|token| {
                parser.expected(
                    token,
                    "expected `}` to end or `,` to continue record literal",
                )
            })?;
            if token.token_type == CBracePipe {
                break token;
            }
            elements.push(RecordElement::parse(parser)?);
        };
        Ok(Self {
            start,
            elements,
            end,
        })
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum RecordElement {
    Element(Expression, Expression),
    Spread(Token, Expression),
}

impl RecordElement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        if let Ok(spread) = parser.expect(OpDotDot) {
            let expression = Expression::parse_parameter_list(parser)?;
            Ok(Self::Spread(spread, expression))
        } else {
            let key = Expression::parse_parameter_list(parser)?;
            parser.expect(OpColon).map_err(|token| {
                parser.expected(token, "expected `:` in key value pair of record literal")
            })?;
            let value = Expression::parse_parameter_list(parser)?;
            Ok(Self::Element(key, value))
        }
    }
}
