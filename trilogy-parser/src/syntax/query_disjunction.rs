use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct QueryDisjunction {
    pub implications: Vec<QueryImplication>,
}
