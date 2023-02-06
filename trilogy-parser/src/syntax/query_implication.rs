use super::*;
use crate::Spanned;
use source_span::Span;

#[derive(Clone, Debug)]
pub struct QueryImplication {
    pub condition: Option<Query>,
    pub conjunctions: Vec<QueryConjunction>,
}

impl Spanned for QueryImplication {
    fn span(&self) -> Span {
        match &self.condition {
            Some(condition) => condition.span().union(self.conjunctions.span()),
            None => self.conjunctions.span(),
        }
    }
}
