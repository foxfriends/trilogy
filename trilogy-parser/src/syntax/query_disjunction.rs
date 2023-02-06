use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct QueryDisjunction {
    pub implications: Vec<QueryImplication>,
}
