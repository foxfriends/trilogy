use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternConjunction {
    pub patterns: Vec<SinglePattern>,
}
