use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Handler {
    pub span: Span,
    pub pattern: Pattern,
    pub guard: Expression,
    pub body: Expression,
}

impl Handler {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::Handler) -> Self {
        match ast {
            syntax::Handler::Given(..) => todo!(), // This doesn't really belong here..?
            syntax::Handler::When(handler) => {
                let span = handler.span();
                let effect = Identifier::temporary(analyzer, handler.pattern.span());
                let pattern = Pattern::convert(analyzer, handler.pattern);
                let pattern =
                    Pattern::binding(pattern.span, effect.clone()).and(pattern.span, pattern);
                let guard = handler
                    .guard
                    .map(|ast| Expression::convert(analyzer, ast))
                    .unwrap_or_else(|| Expression::boolean(span, true));
                let body = handler.body.map(|body| match body {
                    syntax::HandlerBody::Block(block) => {
                        Expression::convert_block(analyzer, *block)
                    }
                    syntax::HandlerBody::Expression(expression) => {
                        Expression::convert(analyzer, *expression)
                    }
                });
                let body = match handler.strategy {
                    syntax::HandlerStrategy::Invert(..) => body.unwrap(),
                    syntax::HandlerStrategy::Resume(token) => {
                        let body = body.unwrap();
                        Expression::builtin(token.span, Builtin::Resume)
                            .apply_to(token.span.union(body.span), body)
                    }
                    syntax::HandlerStrategy::Cancel(token) => {
                        let body = body.unwrap();
                        Expression::builtin(token.span, Builtin::Cancel)
                            .apply_to(token.span.union(body.span), body)
                    }
                    syntax::HandlerStrategy::Yield(token) => {
                        Expression::builtin(token.span, Builtin::Resume).apply_to(
                            token.span,
                            Expression::builtin(token.span, Builtin::Yield)
                                .apply_to(token.span, Expression::reference(pattern.span, effect)),
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
            syntax::Handler::Else(handler) => {
                let span = handler.span();
                let else_span = handler.else_token().span;

                let effect = Identifier::temporary(analyzer, else_span);
                let pattern = handler
                    .identifier
                    .map(|id| Pattern::binding(id.span(), Identifier::declare(analyzer, id)))
                    .unwrap_or_else(|| Pattern::wildcard(else_span));
                let pattern = Pattern::binding(else_span, effect.clone())
                    .and(else_span.union(pattern.span), pattern);
                let guard = Expression::boolean(else_span, true);
                let body = handler.body.map(|body| match body {
                    syntax::HandlerBody::Block(block) => {
                        Expression::convert_block(analyzer, *block)
                    }
                    syntax::HandlerBody::Expression(expression) => {
                        Expression::convert(analyzer, *expression)
                    }
                });

                let body = match handler.strategy {
                    syntax::HandlerStrategy::Yield(token) => {
                        Expression::builtin(token.span, Builtin::Resume).apply_to(
                            token.span,
                            Expression::builtin(token.span, Builtin::Yield)
                                .apply_to(token.span, Expression::reference(pattern.span, effect)),
                        )
                    }
                    syntax::HandlerStrategy::Invert(..) => body.unwrap(),
                    syntax::HandlerStrategy::Resume(token) => {
                        let body = body.unwrap();
                        Expression::builtin(token.span, Builtin::Resume)
                            .apply_to(token.span.union(body.span), body)
                    }
                    syntax::HandlerStrategy::Cancel(token) => {
                        let body = body.unwrap();
                        Expression::builtin(token.span, Builtin::Cancel)
                            .apply_to(token.span.union(body.span), body)
                    }
                };
                Self {
                    span,
                    pattern,
                    guard,
                    body,
                }
            }
        }
    }
}
