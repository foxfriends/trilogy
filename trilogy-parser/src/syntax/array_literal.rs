use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An array literal expression.
///
/// ```trilogy
/// [..prefix, 1, 2, 3, ..suffix]
/// ```
#[derive(Clone, Debug)]
pub struct ArrayLiteral {
    pub open_bracket: Token,
    pub elements: Punctuated<ArrayElement>,
    pub close_bracket: Token,
    pub span: Span,
}

impl Spanned for ArrayLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl ArrayLiteral {
    pub(crate) fn new_empty(open_bracket: Token, close_bracket: Token) -> Self {
        Self {
            span: open_bracket.span.union(close_bracket.span),
            open_bracket,
            elements: Punctuated::default(),
            close_bracket,
        }
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_bracket: Token,
        first: ArrayElement,
    ) -> SyntaxResult<Result<Self, ArrayPattern>> {
        let mut elements = Punctuated::init(first);
        if let Ok(close_bracket) = parser.expect(CBrack) {
            return Ok(Ok(Self {
                span: open_bracket.span.union(close_bracket.span),
                open_bracket,
                elements,
                close_bracket,
            }));
        }

        let close_bracket = loop {
            let comma = parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `]` to end or `,` to continue array literal",
                )
            })?;
            if let Ok(end) = parser.expect(CBrack) {
                elements.finish(comma);
                break end;
            };
            match ArrayElement::parse(parser)? {
                Ok(element) => elements.follow(comma, element),
                Err(next) => {
                    return Ok(Err(ArrayPattern::parse_from_expression(
                        parser,
                        open_bracket,
                        elements,
                        next,
                    )?));
                }
            }
            if let Ok(token) = parser.check(KwFor) {
                let error = SyntaxError::new(
                    token.span,
                    "only one element may precede the `for` keyword in a comprehension",
                );
                parser.error(error.clone());
                return Err(error);
            }
            if let Ok(close_bracket) = parser.expect(CBrack) {
                break close_bracket;
            };
        };
        Ok(Ok(Self {
            span: open_bracket.span.union(close_bracket.span),
            open_bracket,
            elements,
            close_bracket,
        }))
    }
}

#[derive(Clone, Debug, Spanned)]
pub enum ArrayElement {
    Element(Expression),
    Spread(Token, Expression),
}

impl ArrayElement {
    pub(crate) fn parse(
        parser: &mut Parser,
    ) -> SyntaxResult<Result<Self, (Option<Token>, Pattern)>> {
        let spread = parser.expect(OpDotDot).ok();
        if let Some(spread) = &spread
            && let Ok(dot) = parser.expect(OpDot)
        {
            parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
        }
        let expression = Expression::parse_or_pattern(parser)?;
        match expression {
            Ok(expression) => match spread {
                None => Ok(Ok(Self::Element(expression))),
                Some(spread) => Ok(Ok(Self::Spread(spread, expression))),
            },
            Err(pattern) => Ok(Err((spread, pattern))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraylit_empty: "[]" => Expression::parse => Expression::Array(ArrayLiteral { .. }));
    test_parse!(arraylit_one: "[1]" => Expression::parse => Expression::Array(ArrayLiteral { .. }));
    test_parse!(arraylit_one_tc: "[1, ]" => Expression::parse => Expression::Array(ArrayLiteral { .. }));
    test_parse!(arraylit_many: "[1, 2, 3, ]" => Expression::parse => Expression::Array(ArrayLiteral { elements: Punctuated { elements: [_, _, _], .. }, .. }));
    test_parse!(arraylit_many_tc: "[1, 2, 3, ]" => Expression::parse => Expression::Array(ArrayLiteral { elements: Punctuated { elements: [_, _, _], .. }, .. }));
    test_parse!(arraylit_nested: "[[1, 2], [3, 4], [5, 6], ]" => Expression::parse => Expression::Array(ArrayLiteral { elements: Punctuated { elements: [_, _, _], .. }, .. }));
    test_parse!(arraylit_no_comma: "[f 2]" => Expression::parse => Expression::Array(ArrayLiteral { elements: Punctuated { elements: [], last: Some(ArrayElement::Element(Expression::Application(..))) }, .. }));
    test_parse!(arraylit_spread: "[..a, b, ]" => Expression::parse => Expression::Array(ArrayLiteral { elements: Punctuated { elements: [(ArrayElement::Spread(..), _), (ArrayElement::Element(..), _)], .. }, .. }));

    test_parse_error!(arraylit_empty_tc: "[,]" => Expression::parse);
    test_parse_error!(arraylit_missing_item: "[1,,]" => Expression::parse);
    test_parse_error!(arraylit_missing_end: "[1,2," => Expression::parse);
    test_parse_error!(arraylit_incomplete: "[1, 2" => Expression::parse => "expected `]` to end or `,` to continue array literal");
    test_parse_error!(arraylit_mismatched: "[1, 2)" => Expression::parse => "expected `]` to end or `,` to continue array literal");
}
