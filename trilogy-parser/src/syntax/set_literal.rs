use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct SetLiteral {
    pub open_bracket_pipe: Token,
    pub elements: Vec<SetElement>,
    pub close_bracket_pipe: Token,
    pub span: Span,
}

impl Spanned for SetLiteral {
    fn span(&self) -> Span {
        self.span
    }
}

impl SetLiteral {
    pub(crate) fn new_empty(open_bracket_pipe: Token, close_bracket_pipe: Token) -> Self {
        Self {
            span: open_bracket_pipe.span.union(close_bracket_pipe.span),
            open_bracket_pipe,
            elements: vec![],
            close_bracket_pipe,
        }
    }

    pub(crate) fn parse_rest(
        parser: &mut Parser,
        open_bracket_pipe: Token,
        first: SetElement,
    ) -> SyntaxResult<Result<Self, SetPattern>> {
        let mut elements = vec![first];
        if let Ok(close_bracket_pipe) = parser.expect(CBrackPipe) {
            return Ok(Ok(Self {
                span: open_bracket_pipe.span.union(close_bracket_pipe.span),
                open_bracket_pipe,
                elements,
                close_bracket_pipe,
            }));
        };
        let close_bracket_pipe = loop {
            parser.expect(OpComma).map_err(|token| {
                parser.expected(token, "expected `|]` to end or `,` to continue set literal")
            })?;
            if let Ok(close_bracket_pipe) = parser.expect(CBrackPipe) {
                break close_bracket_pipe;
            };
            match SetElement::parse(parser)? {
                Ok(element) => elements.push(element),
                Err(next) => {
                    return Ok(Err(SetPattern::parse_from_expression(
                        parser,
                        open_bracket_pipe,
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
            if let Ok(close_bracket_pipe) = parser.expect(CBrackPipe) {
                break close_bracket_pipe;
            };
        };
        Ok(Ok(Self {
            span: open_bracket_pipe.span.union(close_bracket_pipe.span),
            open_bracket_pipe,
            elements,
            close_bracket_pipe,
        }))
    }

    pub fn start_token(&self) -> &Token {
        &self.open_bracket_pipe
    }

    pub fn end_token(&self) -> &Token {
        &self.close_bracket_pipe
    }
}

#[derive(Clone, Debug, Spanned)]
pub enum SetElement {
    Element(Expression),
    Spread(Token, Expression),
}

impl SetElement {
    pub(crate) fn parse(
        parser: &mut Parser,
    ) -> SyntaxResult<Result<Self, (Option<Token>, Pattern)>> {
        let spread = parser.expect(OpDotDot).ok();
        if let Some(spread) = &spread
            && let Ok(dot) = parser.expect(OpDot)
        {
            parser.error(ErrorKind::TripleDot { dot: dot.span }.at(spread.span));
        }
        let expression = Expression::parse_or_pattern(parser)?;
        match expression {
            Ok(expression) => match spread {
                None => Ok(Ok(Self::Element(expression))),
                Some(spread) => Ok(Ok(Self::Spread(spread, expression))),
            },
            Err(pattern) => Ok(Err((spread, pattern))),
        }
    }
}
