use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct QueryConjunction {
    pub unifications: Vec<Unification>,
}
