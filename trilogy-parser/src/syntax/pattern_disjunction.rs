use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternDisjunction {
    pub conjunctions: Vec<PatternConjunction>,
}
