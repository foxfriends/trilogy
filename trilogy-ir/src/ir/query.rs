use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct Query {
    span: Span,
    pub value: Value,
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
