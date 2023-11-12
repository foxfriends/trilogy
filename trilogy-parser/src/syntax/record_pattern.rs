use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordPattern {
    start: Token,
    pub elements: Vec<(Pattern, Pattern)>,
    pub rest: Option<Pattern>,
    end: Token,
}

impl RecordPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBracePipe)
            .expect("Caller should have found this");
        Self::parse_elements(parser, start, vec![])
    }

    pub(crate) fn parse_elements(
        parser: &mut Parser,
        start: Token,
        mut elements: Vec<(Pattern, Pattern)>,
    ) -> SyntaxResult<Self> {
        let rest = loop {
            if parser.check(CBracePipe).is_ok() {
                break None;
            };
            if let Ok(spread) = parser.expect(OpDotDot) {
                if let Ok(dot) = parser.expect(OpDot) {
                    parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
                }
                break Some(Pattern::parse(parser)?);
            }
            let key = Pattern::parse(parser)?;
            parser.expect(OpFatArrow).map_err(|token| {
                parser.expected(
                    token,
                    "expected `=>` to separate key and value of record element pattern",
                )
            })?;
            let value = Pattern::parse(parser)?;
            elements.push((key, value));
            if parser.check(CBracePipe).is_ok() {
                break None;
            };
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `,` between record pattern elements")
            })?;
        };

        if let Some(rest) = rest {
            return Self::parse_rest(parser, start, elements, rest);
        }

        let end = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record pattern"))?;

        Ok(Self {
            start,
            elements,
            rest,
            end,
        })
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        start: Token,
        elements: Vec<(Pattern, Pattern)>,
        rest: Pattern,
    ) -> SyntaxResult<Self> {
        // We'll consume this trailing comma anyway as if it was going to work,
        // and report an appropriate error. One of few attempts at smart error
        // handling in this parser so far!
        if let Ok(comma) = parser.expect(OpComma) {
            let Ok(end) = parser.expect(CBracePipe) else {
                let error = SyntaxError::new(
                    comma.span,
                    "a rest (`..`) element must end a record pattern",
                );
                parser.error(error.clone());
                return Err(error);
            };
            parser.error(SyntaxError::new(
                comma.span,
                "no trailing comma is permitted after the rest (`..`) element in a record pattern",
            ));
            return Ok(Self {
                start,
                elements,
                rest: Some(rest),
                end,
            });
        }

        let end = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record pattern"))?;

        Ok(Self {
            start,
            elements,
            rest: Some(rest),
            end,
        })
    }

    pub(super) fn parse_from_expression(
        parser: &mut Parser,
        start: Token,
        elements: Vec<RecordElement>,
        (key, value): (Option<Pattern>, Pattern),
    ) -> SyntaxResult<Self> {
        let (mut elements, rest) = elements
            .into_iter()
            .try_fold((vec![], None::<Pattern>), |(mut elements, mut rest), element| {
                match element {
                    RecordElement::Element(key, value) if rest.is_none() => {
                        elements.push((key.try_into()?, value.try_into()?));
                    },
                    RecordElement::Element(key, value) => {
                        return Err(SyntaxError::new(
                            key.span().union(value.span()),
                            "no elements may follow the rest element of a record pattern, you might have meant this to be an expression",
                        ));
                    },
                    RecordElement::Spread(.., value) if rest.is_none() => {
                        rest = Some(value.try_into()?);
                    },
                    RecordElement::Spread(token, value) => {
                        return Err(SyntaxError::new(
                            token.span.union(value.span()),
                            "a record pattern may contain only one rest element, you might have meant this to be an expression",
                        ));
                    },
                }
                Ok((elements, rest))
            })
            .map_err(|error| {
                parser.error(error.clone());
                error
            })?;
        match (rest, key) {
            (None, Some(key)) => {
                elements.push((key, value));
                Self::parse_elements(parser, start, elements)
            }
            (None, None) => {
                Self::parse_rest(parser, start, elements, value)
            }
            (Some(..), Some(key)) => {
                Err(SyntaxError::new(
                    key.span().union(value.span()),
                    "no elements may follow the rest element record a set pattern, you might have meant this to be an expression",
                ))
            }
            (Some(..), None) => {
                Err(SyntaxError::new(
                    value.span(),
                    "a record pattern may contain only one rest element, you might have meant this to be an expression",
                ))
            }
        }
    }

    pub fn start_token(&self) -> &Token {
        &self.start
    }

    pub fn end_token(&self) -> &Token {
        &self.end
    }
}

impl Spanned for RecordPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}

impl TryFrom<RecordLiteral> for RecordPattern {
    type Error = SyntaxError;

    fn try_from(value: RecordLiteral) -> Result<Self, Self::Error> {
        let mut head = vec![];
        let mut rest = None;

        for element in value.elements {
            match element {
                RecordElement::Element(key, val) if rest.is_none() => {
                    head.push((key.try_into()?, val.try_into()?))
                }
                RecordElement::Spread(_, val) if rest.is_none() => rest = Some(val.try_into()?),
                RecordElement::Element(..) | RecordElement::Spread(..) => {
                    return Err(SyntaxError::new(
                        element.span(),
                        "no elements may follow the rest (`..`) element in a set pattern",
                    ))
                }
            }
        }

        Ok(Self {
            start: value.start,
            elements: head,
            rest,
            end: value.end,
        })
    }
}
