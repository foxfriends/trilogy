use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct QueryConjunction {
    pub unifications: Vec<Unification>,
}
