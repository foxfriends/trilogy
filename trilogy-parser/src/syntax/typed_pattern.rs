use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct TypedPattern {
    pub pattern: Pattern,
    pub type_pattern: TypePattern,
}
