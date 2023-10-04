use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ArrayPattern {
    start: Token,
    pub head: Vec<Pattern>,
    pub rest: Option<Pattern>,
    pub tail: Vec<Pattern>,
    end: Token,
}

impl ArrayPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrack)
            .expect("Caller should have found this");

        let mut head = vec![];
        let rest = loop {
            if parser.check(CBrack).is_ok() {
                break None;
            };
            if parser.expect(OpDotDot).is_ok() {
                break Some(Pattern::parse(parser)?);
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

        let mut tail = vec![];
        // at this point, either we:
        // *   saw the `]`, so there will be no rest; or
        // *   parsed a rest pattern, so there must be a comma next before allowing
        //     more elements, or there is no comma so we must be at end of array.
        if rest.is_some() && parser.expect(OpComma).is_ok() {
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
            start,
            head,
            rest,
            tail,
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
                ArrayElement::Spread(.., val) if rest.is_none() => rest = Some(val.try_into()?),
                ArrayElement::Spread(.., val) => {
                    return Err(SyntaxError::new(
                        val.span(),
                        "an array pattern may contain only a single spread element",
                    ))
                }
            }
        }

        Ok(Self {
            start: value.start,
            head,
            rest,
            tail,
            end: value.end,
        })
    }
}

impl Spanned for ArrayPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(arraypat_empty: "[]" => Pattern::parse => "(Pattern::Array (ArrayPattern [] () []))");
    test_parse!(arraypat_one: "[1]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_] () []))");
    test_parse!(arraypat_one_tc: "[1, ]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_] () []))");
    test_parse!(arraypat_many: "[1, 2, 3]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_ _ _] () []))");
    test_parse!(arraypat_many_tc: "[1, 2, 3, ]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_ _ _] () []))");
    test_parse!(arraypat_spread_middle: "[1, 2, ..a, 4, 5]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_ _] (Pattern::Binding _) [_ _]))");
    test_parse!(arraypat_spread_end: "[1, 2, ..a]" => Pattern::parse => "(Pattern::Array (ArrayPattern [_ _] (Pattern::Binding _) []))");
    test_parse!(arraypat_spread_start: "[..a, 1, 2]" => Pattern::parse => "(Pattern::Array (ArrayPattern [] (Pattern::Binding _) [_ _]))");

    test_parse_error!(arraypat_spread_multi: "[..a, 1, ..b]" => Pattern::parse => "array patterns may contain at most one rest (`..`) segment");
    test_parse_error!(arraypat_expression: "[f 2]" => Pattern::parse);
    test_parse_error!(arraypat_empty_tc: "[,]" => Pattern::parse);
    test_parse_error!(arraypat_missing_item: "[1,,]" => Pattern::parse);
    test_parse_error!(arraypat_missing_end: "[1,2," => Pattern::parse);
    test_parse_error!(arraypat_incomplete: "[1, 2" => Pattern::parse => "expected `]` to end or `,` to continue array pattern");
    test_parse_error!(arraypat_mismatched: "[1, 2)" => Pattern::parse => "expected `]` to end or `,` to continue array pattern");
}
