use super::*;

#[derive(Clone, Debug)]
pub struct PatternDisjunction {
    pub conjunctions: Vec<PatternConjunction>,
}
