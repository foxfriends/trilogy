use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct PatternDisjunction {
    pub conjunctions: Vec<PatternConjunction>,
}
