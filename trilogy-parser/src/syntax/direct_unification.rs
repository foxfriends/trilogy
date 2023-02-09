use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct DirectUnification {
    pub pattern: Pattern,
    pub expression: Expression,
}
