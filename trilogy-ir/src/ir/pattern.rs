use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Pattern {
    pub span: Span,
    pub value: Value,
}

impl Pattern {
    pub(super) fn convert(_analyzer: &mut Analyzer, _ast: syntax::Pattern) -> Self {
        todo!()
    }

    pub(super) fn convert_binding(analyzer: &mut Analyzer, ast: syntax::Identifier) -> Self {
        let span = ast.span();
        let id = Identifier::declare(analyzer, ast);
        Self::binding(span, id)
    }

    pub(super) fn binding(span: Span, id: Identifier) -> Self {
        Self {
            span,
            value: Value::Binding(Box::new(id)),
        }
    }

    pub(super) fn wildcard(span: Span) -> Self {
        Self {
            span,
            value: Value::Wildcard,
        }
    }

    pub(super) fn and(self, span: Span, pattern: Pattern) -> Self {
        Self {
            span,
            value: Value::Conjunction(Box::new((self, pattern))),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Conjunction(Box<(Pattern, Pattern)>),
    Disjunction(Box<(Pattern, Pattern)>),
    Mapping(Box<(Pattern, Pattern)>),
    Number(Box<Number>),
    Character(char),
    String(String),
    Bits(Bits),
    Boolean(bool),
    Unit,
    Atom(String),
    Negative(Box<Pattern>),
    Glue(Box<GluePattern>),
    Struct(Box<StructPattern>),
    Tuple(Box<(Pattern, Pattern)>),
    Array(Box<ArrayPattern>),
    Set(Box<SetPattern>),
    Record(Box<RecordPattern>),
    Pinned(Box<Identifier>),
    Binding(Box<Identifier>),
    Wildcard,
}
