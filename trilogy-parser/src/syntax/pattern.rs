use super::*;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub disjunction: PatternDisjunction,
}

#[derive(Clone, Debug)]
pub struct PatternDisjunction {
    pub conjunctions: Vec<PatternConjunction>,
}

#[derive(Clone, Debug)]
pub struct PatternConjunction {
    pub patterns: Vec<SinglePattern>,
}

#[derive(Clone, Debug)]
pub struct SinglePattern {
    pub value_pattern: ValuePattern,
    pub type_pattern: Option<TypePattern>,
}
