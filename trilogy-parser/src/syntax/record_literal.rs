use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordLiteral {
    start: Token,
    pub elements: Vec<RecordElement>,
    end: Token,
}

impl Spanned for RecordLiteral {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

impl RecordLiteral {
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
        first: RecordElement,
    ) -> SyntaxResult<Self> {
        let mut elements = vec![first];
        if let Ok(end) = parser.expect(CBracePipe) {
            return Ok(Self {
                start,
                elements,
                end,
            });
        };
        let end = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `|}` to end or `,` to continue record literal",
                )
            })?;
            if let Ok(end) = parser.expect(CBracePipe) {
                break end;
            };
            elements.push(RecordElement::parse(parser)?);
            if let Ok(token) = parser.check(KwFor) {
                let error = SyntaxError::new(
                    token.span,
                    "only one element may precede the `for` keyword in a comprehension",
                );
                parser.error(error.clone());
                return Err(error);
            }
            if let Ok(end) = parser.expect(CBracePipe) {
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
            parser.expect(OpFatArrow).map_err(|token| {
                parser.expected(token, "expected `=>` in key value pair of record literal")
            })?;
            let value = Expression::parse_parameter_list(parser)?;
            Ok(Self::Element(key, value))
        }
    }
}
