use super::*;

#[derive(Clone, Debug)]
pub struct Query {
    pub disjunction: Vec<QueryDisjunction>,
}
