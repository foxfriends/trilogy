use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Path {
    pub modules: Vec<ModuleReference>,
    pub identifier: Identifier,
}
