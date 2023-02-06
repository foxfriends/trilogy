use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}
