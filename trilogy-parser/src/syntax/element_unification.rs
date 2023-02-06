use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}
