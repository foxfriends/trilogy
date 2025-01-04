use super::*;
use crate::Converter;
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
    fn convert_strategy(
        converter: &mut Converter,
        strategy: syntax::HandlerStrategy,
        effect: Identifier,
    ) -> Expression {
        match strategy {
            syntax::HandlerStrategy::Cancel { body, cancel } => {
                Expression::builtin(cancel.span, Builtin::Cancel)
                    .apply_to(cancel.span, Expression::convert(converter, body))
            }
            syntax::HandlerStrategy::Resume { body, resume } => {
                Expression::builtin(resume.span, Builtin::Cancel).apply_to(
                    resume.span,
                    Expression::builtin(resume.span, Builtin::Resume).apply_to(
                        resume.span.union(body.span()),
                        Expression::convert(converter, body),
                    ),
                )
            }
            syntax::HandlerStrategy::Then { body, .. } => {
                Expression::convert_block(converter, body)
            }
            syntax::HandlerStrategy::Yield(token) => {
                Expression::builtin(token.span, Builtin::Cancel).apply_to(
                    token.span,
                    Expression::builtin(token.span, Builtin::Resume).apply_to(
                        token.span,
                        Expression::builtin(token.span, Builtin::Yield)
                            .apply_to(token.span, Expression::reference(token.span, effect)),
                    ),
                )
            }
        }
    }

    fn convert(converter: &mut Converter, ast: syntax::Handler) -> Self {
        let span = ast.span();
        converter.push_scope();
        let result = match ast {
            syntax::Handler::When(handler) => {
                let effect = Identifier::temporary(converter, handler.pattern.span());
                let pattern = Expression::convert_pattern(converter, handler.pattern);
                let pattern =
                    Expression::reference(pattern.span, effect.clone()).and(pattern.span, pattern);
                let guard = handler
                    .guard
                    .map(|ast| Expression::convert(converter, ast))
                    .unwrap_or_else(|| Expression::boolean(span, true));
                let body = Self::convert_strategy(converter, handler.strategy, effect);
                Self {
                    span,
                    pattern,
                    guard,
                    body,
                }
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
                let body = Self::convert_strategy(converter, handler.strategy, effect);
                Self {
                    span,
                    pattern,
                    guard,
                    body,
                }
            }
        };
        converter.pop_scope();
        result
    }

    pub(super) fn convert_expressions(converter: &mut Converter, ast: syntax::Handler) -> Self {
        Self::convert(converter, ast)
    }
}
