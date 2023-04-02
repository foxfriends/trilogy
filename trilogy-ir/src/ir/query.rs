use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Query {
    span: Span,
    pub value: Value,
}

impl Query {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::Query) -> Self {
        use syntax::Query::*;
        match ast {
            Disjunction(ast) => Self::disjunction(
                ast.span(),
                Self::convert(analyzer, ast.lhs),
                Self::convert(analyzer, ast.rhs),
            ),
            Conjunction(ast) => Self::conjunction(
                ast.span(),
                Self::convert(analyzer, ast.lhs),
                Self::convert(analyzer, ast.rhs),
            ),
            Implication(ast) => Self::implication(
                ast.span(),
                Self::convert(analyzer, ast.lhs),
                Self::convert(analyzer, ast.rhs),
            ),
            Alternative(ast) => Self::alternative(
                ast.span(),
                Self::convert(analyzer, ast.lhs),
                Self::convert(analyzer, ast.rhs),
            ),
            Direct(ast) => Self::direct(ast.span(), Unification::convert_direct(analyzer, *ast)),
            Element(ast) => Self::direct(ast.span(), Unification::convert_element(analyzer, *ast)),
            Parenthesized(ast) => Self::convert(analyzer, ast.query),
            Lookup(ast) => Self::lookup(ast.span(), crate::ir::Lookup::convert(analyzer, *ast)),
            Pass(token) => Self::pass(token.span()),
            End(token) => Self::end(token.span()),
            Is(ast) => Self::is(ast.span(), Expression::convert(analyzer, ast.expression)),
            Not(ast) => Self::not(ast.span(), Self::convert(analyzer, ast.query)),
        }
    }

    pub(super) fn new(span: Span, value: Value) -> Self {
        Self { span, value }
    }

    pub(super) fn not(span: Span, query: Query) -> Self {
        Self::new(span, Value::Not(Box::new(query)))
    }

    pub(super) fn is(span: Span, expression: Expression) -> Self {
        Self::new(span, Value::Is(Box::new(expression)))
    }

    pub(super) fn direct(span: Span, unification: Unification) -> Self {
        Self::new(span, Value::Direct(Box::new(unification)))
    }

    pub(super) fn element(span: Span, unification: Unification) -> Self {
        Self::new(span, Value::Element(Box::new(unification)))
    }

    pub(super) fn disjunction(span: Span, lhs: Query, rhs: Query) -> Self {
        Self::new(span, Value::Disjunction(Box::new((lhs, rhs))))
    }

    pub(super) fn conjunction(span: Span, lhs: Query, rhs: Query) -> Self {
        Self::new(span, Value::Conjunction(Box::new((lhs, rhs))))
    }

    pub(super) fn implication(span: Span, lhs: Query, rhs: Query) -> Self {
        Self::new(span, Value::Implication(Box::new((lhs, rhs))))
    }

    pub(super) fn alternative(span: Span, lhs: Query, rhs: Query) -> Self {
        Self::new(span, Value::Alternative(Box::new((lhs, rhs))))
    }

    pub(super) fn lookup(span: Span, lookup: Lookup) -> Self {
        Self::new(span, Value::Lookup(Box::new(lookup)))
    }

    pub(super) fn pass(span: Span) -> Self {
        Self::new(span, Value::Pass)
    }

    pub(super) fn end(span: Span) -> Self {
        Self::new(span, Value::End)
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Disjunction(Box<(Query, Query)>),
    Conjunction(Box<(Query, Query)>),
    Implication(Box<(Query, Query)>),
    Alternative(Box<(Query, Query)>),
    Direct(Box<Unification>),
    Element(Box<Unification>),
    Lookup(Box<Lookup>),
    Is(Box<Expression>),
    Not(Box<Query>),
    Pass,
    End,
}
