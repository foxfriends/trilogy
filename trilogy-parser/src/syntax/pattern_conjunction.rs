use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct PatternConjunction {
    pub lhs: Pattern,
    pub rns: Pattern,
}
