use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ArrayLiteral {
    start: Token,
    pub elements: Vec<ArrayElement>,
    end: Token,
}

impl ArrayLiteral {
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
        first: ArrayElement,
    ) -> SyntaxResult<Self> {
        let mut elements = vec![first];
        parser.expect(OpComma).map_err(|token| {
            parser.expected(
                token,
                "expected `]` to end or `,` to continue array literal",
            )
        })?;
        let end = loop {
            if let Ok(end) = parser.expect(CBrack) {
                break end;
            };
            elements.push(ArrayElement::parse(parser)?);
            if let Ok(end) = parser.expect(CBrack) {
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
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `]` to end or `,` to continue array literal",
                )
            })?;
        };
        Ok(Self {
            start,
            elements,
            end,
        })
    }
}

impl Spanned for ArrayLiteral {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum ArrayElement {
    Element(Expression),
    Spread(Token, Expression),
}

impl ArrayElement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let spread = parser.expect(OpDotDot).ok();
        let expression = Expression::parse_parameter_list(parser)?;
        match spread {
            None => Ok(Self::Element(expression)),
            Some(spread) => Ok(Self::Spread(spread, expression)),
        }
    }
}
