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
            .expect(OBrace)
            .expect("Caller should have found this");

        let mut elements = vec![];
        let rest = loop {
            if parser.check(CBrace).is_ok() {
                break None;
            };
            if parser.expect(OpDotDot).is_ok() {
                break Some(Pattern::parse(parser)?);
            }
            let key = Pattern::parse(parser)?;
            parser.expect(OpColon).map_err(|token| {
                parser.expected(
                    token,
                    "expected `:` to separate key and value of record element pattern",
                )
            })?;
            let value = Pattern::parse(parser)?;
            elements.push((key, value));
            if parser.check(CBrace).is_ok() {
                break None;
            };
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `,` between record pattern elements")
            })?;
        };

        // We'll consume this trailing comma anyway as if it was going to work,
        // and report an appropriate error. One of few attempts at smart error
        // handling in this parser so far!
        if let Ok(comma) = parser.expect(OpComma) {
            let Ok(end) = parser.expect(CBrace) else {
                let error = SyntaxError::new(
                    comma.span,
                    "a rest (`..`) element must end a record literal",
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
                rest,
                end,
            });
        }

        let end = parser
            .expect(CBrace)
            .map_err(|token| parser.expected(token, "expected `|}` to end record pattern"))?;

        Ok(Self {
            start,
            elements,
            rest,
            end,
        })
    }
}

impl Spanned for RecordPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
