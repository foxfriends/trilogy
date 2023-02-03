use super::*;

#[derive(Clone, Debug)]
pub struct Query {
    pub disjunction: Vec<QueryDisjunction>,
}

#[derive(Clone, Debug)]
pub struct QueryImplication {
    pub condition: Option<Query>,
    pub conjunctions: Vec<QueryConjunction>,
}

#[derive(Clone, Debug)]
pub struct QueryDisjunction {
    pub implications: Vec<QueryImplication>,
}

#[derive(Clone, Debug)]
pub struct QueryConjunction {
    pub unifications: Vec<Unification>,
}
