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
                parser.expected(token, "expected `,` between array pattern elements")
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
                    parser.expected(token, "expected `,` between array pattern elements")
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
}

impl Spanned for ArrayPattern {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span)
    }
}
