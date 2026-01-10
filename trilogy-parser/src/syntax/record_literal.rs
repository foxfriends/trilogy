use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct RecordLiteral {
    pub open_brace_pipe: Token,
    pub elements: Vec<RecordElement>,
    pub close_brace_pipe: Token,
    pub span: Span,
}

impl Spanned for RecordLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl RecordLiteral {
    pub(crate) fn new_empty(open_brace_pipe: Token, close_brace_pipe: Token) -> Self {
        Self {
            span: open_brace_pipe.span.union(close_brace_pipe.span),
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
                span: open_brace_pipe.span.union(close_brace_pipe.span),
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
            span: open_brace_pipe.span.union(close_brace_pipe.span),
            open_brace_pipe,
            elements,
            close_brace_pipe,
        }))
    }
}

#[derive(Clone, Debug)]
pub enum RecordElement {
    Element {
        key: Expression,
        arrow: Token,
        value: Expression,
        span: Span,
    },
    Spread {
        spread: Token,
        value: Expression,
        span: Span,
    },
}

impl Spanned for RecordElement {
    fn span(&self) -> Span {
        match self {
            Self::Element { span, .. } => *span,
            Self::Spread { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RecordPatternElement {
    Element {
        key: Pattern,
        arrow: Token,
        value: Pattern,
        span: Span,
    },
    Spread {
        spread: Token,
        value: Pattern,
        span: Span,
    },
}

impl Spanned for RecordPatternElement {
    fn span(&self) -> Span {
        match self {
            Self::Element { span, .. } => *span,
            Self::Spread { span, .. } => *span,
        }
    }
}

impl RecordElement {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Result<Self, RecordPatternElement>> {
        if let Ok(spread) = parser.expect(OpDotDot) {
            if let Ok(dot) = parser.expect(OpDot) {
                parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
            }

            let expression = Expression::parse_or_pattern(parser)?;
            match expression {
                Ok(expression) => Ok(Ok(Self::Spread {
                    span: spread.span.union(expression.span()),
                    spread,
                    value: expression,
                })),
                Err(pattern) => Ok(Err(RecordPatternElement::Spread {
                    span: spread.span.union(pattern.span()),
                    spread,
                    value: pattern,
                })),
            }
        } else {
            let key = Expression::parse_or_pattern(parser)?;
            let arrow = parser.expect(OpFatArrow).map_err(|token| {
                parser.expected(token, "expected `=>` in key value pair of record literal")
            })?;
            let value = Expression::parse_or_pattern(parser)?;
            match (key, value) {
                (Ok(key), Ok(value)) => Ok(Ok(Self::Element {
                    span: key.span().union(value.span()),
                    key,
                    arrow,
                    value,
                })),
                (Ok(key), Err(value)) => {
                    let key: Pattern = key.try_into()?;
                    Ok(Err(RecordPatternElement::Element {
                        span: key.span().union(value.span()),
                        key,
                        arrow,
                        value,
                    }))
                }
                (Err(key), Ok(value)) => {
                    let value: Pattern = value.try_into()?;
                    Ok(Err(RecordPatternElement::Element {
                        span: key.span().union(value.span()),
                        key,
                        arrow,
                        value,
                    }))
                }
                (Err(key), Err(value)) => Ok(Err(RecordPatternElement::Element {
                    span: key.span().union(value.span()),
                    key,
                    arrow,
                    value,
                })),
            }
        }
    }
}
