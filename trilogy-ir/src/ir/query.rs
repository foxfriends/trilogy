use super::*;
use crate::Converter;
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Query {
    pub span: Span,
    pub value: Value,
}

impl Query {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::Query) -> Self {
        use syntax::Query::*;
        match ast {
            Disjunction(ast) => {
                let span = ast.span();
                let lhs = Self::convert(converter, ast.lhs);
                let rhs = Self::convert(converter, ast.rhs);
                // NOTE: checking for disjoint bindings here is not
                // correct, as variables may be used in the pattern
                // more than once, and then not used in the body,
                // in which case it is ok to be used only in one branch
                // of the query.
                //
                // Will have to do a smarter check elsewhere.
                Self::disjunction(span, lhs, rhs)
            }
            Conjunction(ast) => Self::conjunction(
                ast.span(),
                Self::convert(converter, ast.lhs),
                Self::convert(converter, ast.rhs),
            ),
            Implication(ast) => Self::implication(
                ast.span(),
                Self::convert(converter, ast.lhs),
                Self::convert(converter, ast.rhs),
            ),
            Alternative(ast) => {
                let span = ast.span();
                let lhs = Self::convert(converter, ast.lhs);
                let rhs = Self::convert(converter, ast.rhs);
                Self::alternative(span, lhs, rhs)
            }
            Direct(ast) => Self::direct(ast.span(), Unification::convert_direct(converter, *ast)),
            Element(ast) => {
                Self::element(ast.span(), Unification::convert_element(converter, *ast))
            }
            Parenthesized(ast) => Self::convert(converter, ast.query),
            Lookup(ast) => Self::lookup(ast.span(), crate::ir::Lookup::convert(converter, *ast)),
            Pass(token) => Self::pass(token.span()),
            End(token) => Self::end(token.span()),
            Is(ast) => Self::is(ast.span(), Expression::convert(converter, ast.expression)),
            Not(ast) => {
                converter.push_pseudo_scope();
                let result = Self::not(ast.span(), Self::convert(converter, ast.query));
                converter.pop_scope();
                result
            }
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

    pub fn is_once(&self) -> bool {
        self.value.is_once()
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

impl Value {
    fn is_once(&self) -> bool {
        match self {
            Self::Disjunction(..) => false,
            Self::Conjunction(inner) => inner.0.is_once() && inner.1.is_once(),
            Self::Implication(..) => false, // TODO: these ones might be once?
            Self::Alternative(..) => false, // TODO: these ones might be once?
            Self::Direct(..) => true,
            Self::Pass => true,
            Self::Is(..) => true,
            Self::Lookup(..) => false,
            Self::End => true,
            Self::Not(inner) => inner.is_once(),
            Self::Element(..) => false,
        }
    }
}
