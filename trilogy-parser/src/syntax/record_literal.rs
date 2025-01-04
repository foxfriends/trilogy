use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct RecordLiteral {
    pub open_brace_pipe: Token,
    pub elements: Vec<RecordElement>,
    pub close_brace_pipe: Token,
}

impl Spanned for RecordLiteral {
    fn span(&self) -> Span {
        self.open_brace_pipe.span.union(self.close_brace_pipe.span)
    }
}

impl RecordLiteral {
    pub(crate) fn new_empty(open_brace_pipe: Token, close_brace_pipe: Token) -> Self {
        Self {
            open_brace_pipe,
            elements: vec![],
            close_brace_pipe,
        }
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_brace_pipe: Token,
        first: RecordElement,
    ) -> SyntaxResult<Result<Self, RecordPattern>> {
        let mut elements = vec![first];
        if let Ok(close_brace_pipe) = parser.expect(CBracePipe) {
            return Ok(Ok(Self {
                open_brace_pipe,
                elements,
                close_brace_pipe,
            }));
        };
        let close_brace_pipe = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(
                    token,
                    "expected `|}` to end or `,` to continue record literal",
                )
            })?;
            if let Ok(close_brace_pipe) = parser.expect(CBracePipe) {
                break close_brace_pipe;
            };
            let element = RecordElement::parse(parser)?;
            match element {
                Ok(element) => elements.push(element),
                Err(next) => {
                    return Ok(Err(RecordPattern::parse_from_expression(
                        parser,
                        open_brace_pipe,
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
            if let Ok(close_brace_pipe) = parser.expect(CBracePipe) {
                break close_brace_pipe;
            };
        };
        Ok(Ok(Self {
            open_brace_pipe,
            elements,
            close_brace_pipe,
        }))
    }
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum RecordElement {
    Element(Expression, Expression),
    Spread(Token, Expression),
}

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum RecordPatternElement {
    Element(Pattern, Pattern),
    Spread(Token, Pattern),
}

impl RecordElement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Result<Self, RecordPatternElement>> {
        if let Ok(spread) = parser.expect(OpDotDot) {
            if let Ok(dot) = parser.expect(OpDot) {
                parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
            }

            let expression = Expression::parse_parameter_list(parser)?;
            match expression {
                Ok(expression) => Ok(Ok(Self::Spread(spread, expression))),
                Err(pattern) => Ok(Err(RecordPatternElement::Spread(spread, pattern))),
            }
        } else {
            let key = Expression::parse_parameter_list(parser)?;
            parser.expect(OpFatArrow).map_err(|token| {
                parser.expected(token, "expected `=>` in key value pair of record literal")
            })?;
            let value = Expression::parse_parameter_list(parser)?;
            match (key, value) {
                (Ok(key), Ok(value)) => Ok(Ok(Self::Element(key, value))),
                (Ok(key), Err(value)) => {
                    Ok(Err(RecordPatternElement::Element(key.try_into()?, value)))
                }
                (Err(key), Ok(value)) => {
                    Ok(Err(RecordPatternElement::Element(key, value.try_into()?)))
                }
                (Err(key), Err(value)) => Ok(Err(RecordPatternElement::Element(key, value))),
            }
        }
    }
}
