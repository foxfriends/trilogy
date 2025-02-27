use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct SetPattern {
    pub open_bracket_pipe: Token,
    pub elements: Vec<Pattern>,
    pub rest: Option<RestPattern>,
    pub close_bracket_pipe: Token,
}

impl SetPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let open_bracket_pipe = parser.expect(OBrackPipe).unwrap();
        Self::parse_elements(parser, open_bracket_pipe, vec![])
    }

    pub(crate) fn parse_elements(
        parser: &mut Parser,
        open_bracket_pipe: Token,
        mut elements: Vec<Pattern>,
    ) -> SyntaxResult<Self> {
        let rest = loop {
            if parser.check(CBrackPipe).is_ok() {
                break None;
            };
            if let Ok(spread) = parser.expect(OpDotDot) {
                if let Ok(dot) = parser.expect(OpDot) {
                    parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
                }
                break Some(RestPattern::parse(parser, spread)?);
            }
            elements.push(Pattern::parse(parser)?);
            if parser.check(CBrackPipe).is_ok() {
                break None;
            };
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `,` between set pattern elements")
            })?;
        };

        if let Some(rest) = rest {
            return Self::parse_rest(parser, open_bracket_pipe, elements, rest);
        }
        let close_bracket_pipe = parser
            .expect(CBrackPipe)
            .map_err(|token| parser.expected(token, "expected `|]` to end set pattern"))?;

        Ok(Self {
            open_bracket_pipe,
            elements,
            rest,
            close_bracket_pipe,
        })
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_bracket_pipe: Token,
        elements: Vec<Pattern>,
        rest: RestPattern,
    ) -> SyntaxResult<Self> {
        // We'll consume this trailing comma anyway as if it was going to work,
        // and report an appropriate error. One of few attempts at smart error
        // handling in this parser so far!
        if let Ok(comma) = parser.expect(OpComma) {
            let Ok(close_bracket_pipe) = parser.expect(CBrackPipe) else {
                let error = SyntaxError::new(
                    comma.span,
                    "a rest (`..`) element must close_bracket_pipe a set pattern",
                );
                parser.error(error.clone());
                return Err(error);
            };
            parser.error(SyntaxError::new(
                comma.span,
                "no trailing comma is permitted after the rest (`..`) element in a set pattern",
            ));
            return Ok(Self {
                open_bracket_pipe,
                elements,
                rest: Some(rest),
                close_bracket_pipe,
            });
        }
        let close_bracket_pipe = parser.expect(CBrackPipe).map_err(|token| {
            parser.expected(token, "expected `|]` to close_bracket_pipe set pattern")
        })?;
        Ok(Self {
            open_bracket_pipe,
            elements,
            rest: Some(rest),
            close_bracket_pipe,
        })
    }

    pub(crate) fn parse_from_expression(
        parser: &mut Parser,
        open_bracket_pipe: Token,
        elements: Vec<SetElement>,
        (spread, next): (Option<Token>, Pattern),
    ) -> SyntaxResult<Self> {
        let (mut elements, rest) = elements
            .into_iter()
            .try_fold(
                (vec![], None::<Pattern>),
                |(mut elements, mut spread), element| {
                match element {
                    SetElement::Element(element) if spread.is_none() => {
                        elements.push(element.try_into()?)
                    }
                    SetElement::Element(element) => {
                        return Err(SyntaxError::new(
                            element.span(),
                            "no elements may follow the rest element of a set pattern, you might have meant this to be an expression",
                        ));
                    }
                    SetElement::Spread(_, element) if spread.is_none() => {
                        spread = Some(element.try_into()?);
                    }
                    SetElement::Spread(token, element) => {
                        return Err(SyntaxError::new(
                            token.span.union(element.span()),
                            "a set pattern may contain only one rest element, you might have meant this to be an expression",
                        ));
                    }
                }
                Ok((elements, spread))
            })
            .inspect_err(|error| {
                parser.error(error.clone());
            })?;
        match rest {
            None if spread.is_none() => {
                elements.push(next);
                Self::parse_elements(parser, open_bracket_pipe, elements)
            }
            None => Self::parse_rest(
                parser,
                open_bracket_pipe,
                elements,
                RestPattern::new(spread.unwrap(), next),
            ),
            Some(..) if spread.is_none() => Err(SyntaxError::new(
                next.span().union(spread.unwrap().span()),
                "no elements may follow the rest element of a set pattern, you might have meant this to be an expression",
            )),
            Some(rest) => Err(SyntaxError::new(
                rest.span().union(spread.unwrap().span()),
                "a set pattern may contain only one rest element, you might have meant this to be an expression",
            )),
        }
    }
}

impl Spanned for SetPattern {
    fn span(&self) -> Span {
        self.open_bracket_pipe
            .span
            .union(self.close_bracket_pipe.span)
    }
}

impl TryFrom<SetLiteral> for SetPattern {
    type Error = SyntaxError;

    fn try_from(value: SetLiteral) -> Result<Self, Self::Error> {
        let mut head = vec![];
        let mut rest = None;

        for element in value.elements {
            match element {
                SetElement::Element(val) if rest.is_none() => head.push(val.try_into()?),
                SetElement::Spread(token, val) if rest.is_none() => {
                    rest = Some(RestPattern::try_from((token, val))?)
                }
                SetElement::Element(val) | SetElement::Spread(_, val) => {
                    return Err(SyntaxError::new(
                        val.span(),
                        "no elements may follow the rest (`..`) element in a set pattern",
                    ));
                }
            }
        }

        Ok(Self {
            open_bracket_pipe: value.open_bracket_pipe,
            elements: head,
            rest,
            close_bracket_pipe: value.close_bracket_pipe,
        })
    }
}
