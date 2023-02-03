use super::*;

#[derive(Clone, Debug)]
pub struct Pattern {
    pub conjunction: PatternConjunction,
}

#[derive(Clone, Debug)]
pub struct PatternConjunction {
    pub disjunctions: Vec<PatternDisjunction>,
}

#[derive(Clone, Debug)]
pub struct PatternDisjunction {
    pub patterns: Vec<SinglePattern>,
}

#[derive(Clone, Debug)]
pub struct SinglePattern {
    pub value_pattern: ValuePattern,
    pub type_pattern: Option<TypePattern>,
}
