use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An array literal expression.
///
/// ```trilogy
/// [1, 2, 3]
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ArrayLiteral {
    pub obrack: Token,
    pub elements: Vec<ArrayElement>,
    pub cbrack: Token,
}

impl ArrayLiteral {
    pub(crate) fn new_empty(obrack: Token, cbrack: Token) -> Self {
        Self {
            obrack,
            elements: vec![],
            cbrack,
        }
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        obrack: Token,
        first: ArrayElement,
    ) -> SyntaxResult<Result<Self, ArrayPattern>> {
        let mut elements = vec![first];
        if let Ok(cbrack) = parser.expect(CBrack) {
            return Ok(Ok(Self {
                obrack,
                elements,
                cbrack,
            }));
        }

        let cbrack = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `]` to end or `,` to continue array literal",
                )
            })?;
            if let Ok(end) = parser.expect(CBrack) {
                break end;
            };
            match ArrayElement::parse(parser)? {
                Ok(element) => elements.push(element),
                Err(next) => {
                    return Ok(Err(ArrayPattern::parse_from_expression(
                        parser, obrack, elements, next,
                    )?))
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
            if let Ok(cbrack) = parser.expect(CBrack) {
                break cbrack;
            };
        };
        Ok(Ok(Self {
            obrack,
            elements,
            cbrack,
        }))
    }
}

impl Spanned for ArrayLiteral {
    fn span(&self) -> Span {
        self.obrack.span.union(self.cbrack.span)
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum ArrayElement {
    Element(Expression),
    Spread(Token, Expression),
}

impl ArrayElement {
    pub(crate) fn parse(
        parser: &mut Parser,
    ) -> SyntaxResult<Result<Self, (Option<Token>, Pattern)>> {
        let spread = parser.expect(OpDotDot).ok();
        if let Some(spread) = &spread {
            if let Ok(dot) = parser.expect(OpDot) {
                parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
            }
        }
        let expression = Expression::parse_parameter_list(parser)?;
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

    test_parse!(arraylit_empty: "[]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [] _))");
    test_parse!(arraylit_one: "[1]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [_] _))");
    test_parse!(arraylit_one_tc: "[1, ]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [_] _))");
    test_parse!(arraylit_many: "[1, 2, 3]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [_ _ _] _))");
    test_parse!(arraylit_many_tc: "[1, 2, 3, ]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [_ _ _] _))");
    test_parse!(arraylit_nested: "[[1, 2], [3, 4], [5, 6]]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [_ _ _] _))");
    test_parse!(arraylit_no_comma: "[f 2]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [(_ (Expression::Application _))] _))");
    test_parse!(arraylit_spread: "[..a, b]" => Expression::parse => "(Expression::Array (ArrayLiteral _ [(ArrayElement::Spread _ _) (ArrayElement::Element _)] _))");

    test_parse_error!(arraylit_empty_tc: "[,]" => Expression::parse);
    test_parse_error!(arraylit_missing_item: "[1,,]" => Expression::parse);
    test_parse_error!(arraylit_missing_end: "[1,2," => Expression::parse);
    test_parse_error!(arraylit_incomplete: "[1, 2" => Expression::parse => "expected `]` to end or `,` to continue array literal");
    test_parse_error!(arraylit_mismatched: "[1, 2)" => Expression::parse => "expected `]` to end or `,` to continue array literal");
}
