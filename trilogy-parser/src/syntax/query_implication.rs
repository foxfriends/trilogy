use super::*;

#[derive(Clone, Debug)]
pub struct QueryImplication {
    pub condition: Option<Query>,
    pub conjunctions: Vec<QueryConjunction>,
}
