use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct SetPattern {
    start: Token,
    pub elements: Vec<Pattern>,
    pub rest: Option<Pattern>,
    end: Token,
}

impl SetPattern {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let start = parser
            .expect(OBrackPipe)
            .expect("Caller should have found this");
        Self::parse_elements(parser, start, vec![])
    }

    pub(crate) fn parse_elements(
        parser: &mut Parser,
        start: Token,
        mut elements: Vec<Pattern>,
    ) -> SyntaxResult<Self> {
        let rest = loop {
            if parser.check(CBrackPipe).is_ok() {
                break None;
            };
            if parser.expect(OpDotDot).is_ok() {
                break Some(Pattern::parse(parser)?);
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
            return Self::parse_rest(parser, start, elements, rest);
        }
        let end = parser
            .expect(CBrackPipe)
            .map_err(|token| parser.expected(token, "expected `|]` to end set pattern"))?;

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
        elements: Vec<Pattern>,
        rest: Pattern,
    ) -> SyntaxResult<Self> {
        // We'll consume this trailing comma anyway as if it was going to work,
        // and report an appropriate error. One of few attempts at smart error
        // handling in this parser so far!
        if let Ok(comma) = parser.expect(OpComma) {
            let Ok(end) = parser.expect(CBrackPipe) else {
                let error =
                    SyntaxError::new(comma.span, "a rest (`..`) element must end a set pattern");
                parser.error(error.clone());
                return Err(error);
            };
            parser.error(SyntaxError::new(
                comma.span,
                "no trailing comma is permitted after the rest (`..`) element in a set pattern",
            ));
            return Ok(Self {
                start,
                elements,
                rest: Some(rest),
                end,
            });
        }
        let end = parser
            .expect(CBrackPipe)
            .map_err(|token| parser.expected(token, "expected `|]` to end set pattern"))?;
        Ok(Self {
            start,
            elements,
            rest: Some(rest),
            end,
        })
    }

    pub(crate) fn parse_from_expression(
        parser: &mut Parser,
        start: Token,
        elements: Vec<SetElement>,
        (spread, next): (Option<Token>, Pattern),
    ) -> SyntaxResult<Self> {
        let (mut elements, rest) = elements
            .into_iter()
            .try_fold(
                (vec![], None),
                |(mut elements, mut spread), element| {
                match element {
                    SetElement::Element(element) if spread.is_none() => {
                        elements.push(element.try_into()?)
                    }
                    SetElement::Element(element) => {
                        return Err(SyntaxError::new(element.span(), "no elements may follow the rest element of a set pattern, you might have meant this to be an expression"));
                    }
                    SetElement::Spread(_, element) if spread.is_none() => {
                        spread = Some(element.try_into()?);
                    }
                    SetElement::Spread(token, element) => {
                        return Err(SyntaxError::new(token.span.union(element.span()), "a set pattern may contain only one rest element, you might have meant this to be an expression"));
                    }
                }
                Ok((elements, spread))
            })
            .map_err(|error| {
                parser.error(error.clone());
                error
            })?;
        match rest {
            None if spread.is_none() => {
                elements.push(next);
                Self::parse_elements(parser, start, elements)
            }
            None => Self::parse_rest(parser, start, elements, next),
            Some(rest) if spread.is_none() => Self::parse_rest(parser, start, elements, rest),
            Some(rest) => {
                return Err(SyntaxError::new(rest.span().union(spread.unwrap().span()), "a set pattern may contain only one rest element, you might have meant this to be an expression"));
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

impl Spanned for SetPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
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
                SetElement::Spread(_, val) if rest.is_none() => rest = Some(val.try_into()?),
                SetElement::Element(val) | SetElement::Spread(_, val) => {
                    return Err(SyntaxError::new(
                        val.span(),
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
