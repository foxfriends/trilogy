use super::*;

#[derive(Clone, Debug)]
pub struct QueryDisjunction {
    pub implications: Vec<QueryImplication>,
}
