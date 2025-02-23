use super::{record_literal::RecordPatternElement, *};
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordPattern {
    pub open_brace_pipe: Token,
    pub elements: Vec<(Pattern, Pattern)>,
    pub rest: Option<RestPattern>,
    pub close_brace_pipe: Token,
}

impl RecordPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let open_brace_pipe = parser.expect(OBracePipe).unwrap();
        Self::parse_elements(parser, open_brace_pipe, vec![])
    }

    pub(crate) fn parse_elements(
        parser: &mut Parser,
        open_brace_pipe: Token,
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
                break Some(RestPattern::parse(parser, spread)?);
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
            return Self::parse_rest(parser, open_brace_pipe, elements, rest);
        }

        let close_brace_pipe = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record pattern"))?;

        Ok(Self {
            open_brace_pipe,
            elements,
            rest,
            close_brace_pipe,
        })
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_brace_pipe: Token,
        elements: Vec<(Pattern, Pattern)>,
        rest: RestPattern,
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
                open_brace_pipe,
                elements,
                rest: Some(rest),
                close_brace_pipe: end,
            });
        }

        let end = parser
            .expect(CBracePipe)
            .map_err(|token| parser.expected(token, "expected `|}` to end record pattern"))?;

        Ok(Self {
            open_brace_pipe,
            elements,
            rest: Some(rest),
            close_brace_pipe: end,
        })
    }

    pub(super) fn parse_from_expression(
        parser: &mut Parser,
        open_brace_pipe: Token,
        elements: Vec<RecordElement>,
        head_element: RecordPatternElement,
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
            .inspect_err(|error| {
                parser.error(error.clone());
            })?;
        match (rest, head_element) {
            (None, RecordPatternElement::Element(key, value)) => {
                elements.push((key, value));
                Self::parse_elements(parser, open_brace_pipe, elements)
            }
            (None, RecordPatternElement::Spread(spread, value)) => Self::parse_rest(
                parser,
                open_brace_pipe,
                elements,
                RestPattern::new(spread, value),
            ),
            (Some(..), element @ RecordPatternElement::Element(..)) => Err(SyntaxError::new(
                element.span(),
                "no elements may follow the rest element in a record pattern, you might have meant this to be an expression",
            )),
            (Some(..), element @ RecordPatternElement::Spread(..)) => Err(SyntaxError::new(
                element.span(),
                "a record pattern may contain only one rest element, you might have meant this to be an expression",
            )),
        }
    }

    pub fn start_token(&self) -> &Token {
        &self.open_brace_pipe
    }

    pub fn end_token(&self) -> &Token {
        &self.close_brace_pipe
    }
}

impl Spanned for RecordPattern {
    fn span(&self) -> Span {
        self.open_brace_pipe.span.union(self.close_brace_pipe.span)
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
                RecordElement::Spread(spread, val) if rest.is_none() => {
                    rest = Some(RestPattern::try_from((spread, val))?)
                }
                RecordElement::Element(..) | RecordElement::Spread(..) => {
                    return Err(SyntaxError::new(
                        element.span(),
                        "no elements may follow the rest (`..`) element in a set pattern",
                    ));
                }
            }
        }

        Ok(Self {
            open_brace_pipe: value.open_brace_pipe,
            elements: head,
            rest,
            close_brace_pipe: value.close_brace_pipe,
        })
    }
}
