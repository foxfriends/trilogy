use super::*;

#[derive(Clone, Debug)]
pub struct QueryConjunction {
    pub unifications: Vec<Unification>,
}
