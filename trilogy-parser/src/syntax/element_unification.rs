use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ElementUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}
