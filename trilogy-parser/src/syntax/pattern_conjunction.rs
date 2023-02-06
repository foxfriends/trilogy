use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct PatternConjunction {
    pub patterns: Vec<SinglePattern>,
}
