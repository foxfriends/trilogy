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
        if let Ok(end) = parser.expect(CBrack) {
            return Ok(Self {
                start,
                elements,
                end,
            });
        }

        let end = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `]` to end or `,` to continue array literal",
                )
            })?;
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

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraylit_empty: "[]" => Expression::parse => "(Expression::Array (ArrayLiteral []))");
    test_parse!(arraylit_one: "[1]" => Expression::parse => "(Expression::Array (ArrayLiteral [_]))");
    test_parse!(arraylit_one_tc: "[1, ]" => Expression::parse => "(Expression::Array (ArrayLiteral [_]))");
    test_parse!(arraylit_many: "[1, 2, 3]" => Expression::parse => "(Expression::Array (ArrayLiteral [_ _ _]))");
    test_parse!(arraylit_many_tc: "[1, 2, 3, ]" => Expression::parse => "(Expression::Array (ArrayLiteral [_ _ _]))");
    test_parse!(arraylit_nested: "[[1, 2], [3, 4], [5, 6]]" => Expression::parse => "(Expression::Array (ArrayLiteral [_ _ _]))");
    test_parse!(arraylit_no_comma: "[f 2]" => Expression::parse => "(Expression::Array (ArrayLiteral [(_ (Expression::Application _))]))");

    test_parse_error!(arraylit_empty_tc: "[,]" => Expression::parse);
    test_parse_error!(arraylit_missing_item: "[1,,]" => Expression::parse);
    test_parse_error!(arraylit_missing_end: "[1,2," => Expression::parse);
    test_parse_error!(arraylit_incomplete: "[1, 2" => Expression::parse => "expected `]` to end or `,` to continue array literal");
    test_parse_error!(arraylit_mismatched: "[1, 2)" => Expression::parse => "expected `]` to end or `,` to continue array literal");
}
