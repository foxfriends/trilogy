use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Query {
    span: Span,
    pub value: Value,
}

impl Query {
    pub(super) fn convert(_analyzer: &mut Analyzer, _query: syntax::Query) -> Self {
        todo!()
    }

    pub(super) fn pass(span: Span) -> Self {
        Self {
            span,
            value: Value::Pass,
        }
    }
}

// TODO: is there a way to turn queries into expressions too?
// maybe not

#[derive(Clone, Debug)]
pub enum Value {
    Disjunction(Box<(Query, Query)>),
    Conjunction(Box<(Query, Query)>),
    Implication(Box<(Query, Query)>),
    Alternative(Box<(Query, Query)>),
    Direct(Box<DirectUnification>),
    Element(Box<ElementUnification>),
    Lookup(Box<Lookup>),
    Is(Box<Expression>),
    Not(Box<Query>),
    Pass,
    End,
}
