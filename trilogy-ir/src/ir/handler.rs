use super::*;
use crate::{Converter, Error};
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Handler {
    pub span: Span,
    pub pattern: Expression,
    pub guard: Expression,
    pub body: Expression,
}

impl Handler {
    fn convert(converter: &mut Converter, ast: syntax::Handler, is_expression: bool) -> Self {
        let span = ast.span();
        let (pattern, guard, body, effect, strategy) = match ast {
            syntax::Handler::When(handler) => {
                let effect = Identifier::temporary(converter, handler.pattern.span());
                let pattern = Expression::convert_pattern(converter, handler.pattern);
                let pattern =
                    Expression::reference(pattern.span, effect.clone()).and(pattern.span, pattern);
                let guard = handler
                    .guard
                    .map(|ast| Expression::convert(converter, ast))
                    .unwrap_or_else(|| Expression::boolean(span, true));
                let body = handler.body.map(|body| match body {
                    syntax::HandlerBody::Block(block) => {
                        if is_expression {
                            converter.error(Error::BlockInExpressionHandler { span: block.span() });
                        }
                        Expression::convert_block(converter, *block)
                    }
                    syntax::HandlerBody::Expression(expression) => {
                        if !is_expression {
                            converter.error(Error::ExpressionInBlockHandler {
                                span: expression.span(),
                            });
                        }
                        Expression::convert(converter, *expression)
                    }
                });
                (pattern, guard, body, effect, handler.strategy)
            }
            syntax::Handler::Else(handler) => {
                let else_span = handler.else_token().span;
                let effect = Identifier::temporary(converter, else_span);
                let pattern = handler
                    .identifier
                    .map(|id| Expression::reference(id.span(), Identifier::declare(converter, id)))
                    .unwrap_or_else(|| Expression::wildcard(else_span));
                let pattern = Expression::reference(else_span, effect.clone())
                    .and(else_span.union(pattern.span), pattern);
                let guard = Expression::boolean(else_span, true);
                let body = handler.body.map(|body| match body {
                    syntax::HandlerBody::Block(block) => {
                        Expression::convert_block(converter, *block)
                    }
                    syntax::HandlerBody::Expression(expression) => {
                        Expression::convert(converter, *expression)
                    }
                });
                (pattern, guard, body, effect, handler.strategy)
            }
        };
        let body = match strategy {
            syntax::HandlerStrategy::Invert(..) => body.unwrap(),
            syntax::HandlerStrategy::Resume(token) => {
                let body = body.unwrap();
                Expression::builtin(token.span, Builtin::Cancel).apply_to(
                    token.span,
                    Expression::builtin(token.span, Builtin::Resume)
                        .apply_to(token.span.union(body.span), body),
                )
            }
            syntax::HandlerStrategy::Cancel(token) => {
                let body = body.unwrap();
                Expression::builtin(token.span, Builtin::Cancel)
                    .apply_to(token.span.union(body.span), body)
            }
            syntax::HandlerStrategy::Yield(token) => {
                Expression::builtin(token.span, Builtin::Cancel).apply_to(
                    token.span,
                    Expression::builtin(token.span, Builtin::Resume).apply_to(
                        token.span,
                        Expression::builtin(token.span, Builtin::Yield)
                            .apply_to(token.span, Expression::reference(pattern.span, effect)),
                    ),
                )
            }
        };
        Self {
            span,
            pattern,
            guard,
            body,
        }
    }

    pub(super) fn convert_blocks(converter: &mut Converter, ast: syntax::Handler) -> Self {
        Self::convert(converter, ast, false)
    }

    pub(super) fn convert_expressions(converter: &mut Converter, ast: syntax::Handler) -> Self {
        Self::convert(converter, ast, true)
    }
}
