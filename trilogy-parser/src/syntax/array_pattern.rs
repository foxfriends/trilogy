use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// An array pattern.
///
/// ```trilogy
/// [1, 2, 3, ..rest, 5, 6, 7]
/// ```
#[derive(Clone, Debug)]
pub struct ArrayPattern {
    pub open_bracket: Token,
    /// The head elements, which come before the optional `rest` element. Will be all elements if `rest` is `None`.
    pub head: Vec<Pattern>,
    /// A single optional spread elements, containing the rest of the elements.
    pub rest: Option<RestPattern>,
    /// The tail elements, which follow the spread. Will be empty if `rest` is `None`.
    pub tail: Vec<Pattern>,
    pub close_bracket: Token,
}

impl ArrayPattern {
    pub(crate) fn parse_elements(
        parser: &mut Parser,
        open_bracket: Token,
        mut head: Vec<Pattern>,
    ) -> SyntaxResult<Self> {
        let rest = loop {
            if parser.check(CBrack).is_ok() {
                break None;
            };
            if let Ok(spread) = parser.expect(OpDotDot) {
                if let Ok(dot) = parser.expect(OpDot) {
                    parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
                }
                break Some(RestPattern::parse(parser, spread)?);
            }
            head.push(Pattern::parse(parser)?);
            if parser.check(CBrack).is_ok() {
                break None;
            };
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `]` to end or `,` to continue array pattern",
                )
            })?;
        };

        if let Some(rest) = rest {
            return Self::parse_rest(parser, open_bracket, head, rest, vec![]);
        }

        let close_bracket = parser
            .expect(CBrack)
            .map_err(|token| parser.expected(token, "expected `]` to end array pattern"))?;

        Ok(Self {
            open_bracket,
            head,
            rest: None,
            tail: vec![],
            close_bracket,
        })
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        head: Vec<Pattern>,
        rest: RestPattern,
        mut tail: Vec<Pattern>,
    ) -> SyntaxResult<Self> {
        // at this point, either we:
        // *   saw the `]`, so there will be no rest; or
        // *   parsed a rest pattern, so there must be a comma next before allowing
        //     more elements, or there is no comma so we must be at end of array.
        if parser.expect(OpComma).is_ok() {
            loop {
                if parser.check(CBrack).is_ok() {
                    break;
                };
                if let Ok(token) = parser.expect(OpDotDot) {
                    // Avoid an error cascade here by parsing the rest pattern as a regular
                    // pattern, discarding the "restness".
                    parser.error(SyntaxError::new(
                        token.span,
                        "array patterns may contain at most one rest (`..`) segment",
                    ));
                }
                tail.push(Pattern::parse(parser)?);
                if parser.check(CBrack).is_ok() {
                    break;
                };
                parser.expect(OpComma).map_err(|token| {
                    parser.expected(
                        token,
                        "expected `]` to end or `,` to continue array pattern",
                    )
                })?;
            }
        }

        let end = parser
            .expect(CBrack)
            .map_err(|token| parser.expected(token, "expected `]` to end array pattern"))?;

        Ok(Self {
            open_bracket: start,
            head,
            rest: Some(rest),
            tail,
            close_bracket: end,
        })
    }

    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrack)
            .expect("Caller should have found this");
        Self::parse_elements(parser, start, vec![])
    }

    pub(crate) fn parse_from_expression(
        parser: &mut Parser,
        start: Token,
        elements: Punctuated<ArrayElement>,
        (spread, next): (Option<Token>, Pattern),
    ) -> SyntaxResult<Self> {
        let (head, rest, tail) = elements.into_iter().try_fold(
            (vec![], None, vec![]),
            |(mut head, mut spread, mut tail), element| {
                match element {
                    ArrayElement::Element(expr) if spread.is_none() => {
                        head.push(expr.try_into()?);
                    }
                    ArrayElement::Element(expr) => tail.push(expr.try_into()?),
                    ArrayElement::Spread(sp, expr) if spread.is_none() => {
                        spread = Some(RestPattern::try_from((sp, expr))?);
                    }
                    ArrayElement::Spread(token, element) => return Err(SyntaxError::new(
                        token.span.union(element.span()),
                        "an array pattern may contain only one rest element, or you might have meant this to be an array expression",
                    )),
                }
                Ok((head, spread, tail))
            },
        ).inspect_err(|error| {
            parser.error(error.clone());
        })?;
        match (spread, rest) {
            (None, None) => Self::parse_elements(parser, start, head),
            (None, Some(rest)) => Self::parse_rest(parser, start, head, rest, tail),
            (Some(token), None) => {
                Self::parse_rest(parser, start, head, RestPattern::new(token, next), tail)
            }
            (Some(token), Some(..)) => {
                let error = SyntaxError::new(
                    token.span.union(next.span()),
                    "an array pattern may only contain a single spread element, or you might have meant this to be an array expression",
                );
                parser.error(error.clone());
                Err(error)
            }
        }
    }
}

impl TryFrom<ArrayLiteral> for ArrayPattern {
    type Error = SyntaxError;

    fn try_from(value: ArrayLiteral) -> Result<Self, Self::Error> {
        let mut head = vec![];
        let mut tail = vec![];
        let mut rest = None;

        for element in value.elements {
            match element {
                ArrayElement::Element(val) if rest.is_none() => head.push(val.try_into()?),
                ArrayElement::Element(val) => tail.push(val.try_into()?),
                ArrayElement::Spread(spread, val) if rest.is_none() => {
                    rest = Some(RestPattern::try_from((spread, val))?)
                }
                ArrayElement::Spread(.., val) => {
                    return Err(SyntaxError::new(
                        val.span(),
                        "an array pattern may contain only a single spread element",
                    ));
                }
            }
        }

        Ok(Self {
            open_bracket: value.open_bracket,
            head,
            rest,
            tail,
            close_bracket: value.close_bracket,
        })
    }
}

impl Spanned for ArrayPattern {
    fn span(&self) -> Span {
        self.open_bracket.span.union(self.close_bracket.span)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraypat_empty: "[]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_one: "[1]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_one_tc: "[1, ]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_many: "[1, 2, 3]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_many_tc: "[1, 2, 3, ]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_spread_middle: "[1, 2, ..a, 4, 5]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_spread_end: "[1, 2, ..a]" => Pattern::parse => Pattern::Array(..));
    test_parse!(arraypat_spread_start: "[..a, 1, 2]" => Pattern::parse => Pattern::Array(..));

    test_parse_error!(arraypat_spread_multi: "[..a, 1, ..b]" => Pattern::parse => "array patterns may contain at most one rest (`..`) segment");
    test_parse_error!(arraypat_expression: "[f 2]" => Pattern::parse);
    test_parse_error!(arraypat_empty_tc: "[,]" => Pattern::parse);
    test_parse_error!(arraypat_missing_item: "[1,,]" => Pattern::parse);
    test_parse_error!(arraypat_missing_end: "[1,2," => Pattern::parse);
    test_parse_error!(arraypat_incomplete: "[1, 2" => Pattern::parse => "expected `]` to end or `,` to continue array pattern");
    test_parse_error!(arraypat_mismatched: "[1, 2)" => Pattern::parse => "expected `]` to end or `,` to continue array pattern");
}
