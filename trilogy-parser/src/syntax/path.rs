use super::*;

#[derive(Clone, Debug, Spanned)]
pub struct Path {
    pub modules: Vec<ModuleReference>,
    pub identifier: Identifier,
}
