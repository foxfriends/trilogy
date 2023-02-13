use super::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct Path {
    pub modules: Vec<ModuleReference>,
    pub identifier: Identifier,
}

impl Path {
    pub(crate) fn new(modules: Vec<ModuleReference>, identifier: Identifier) -> Self {
        Self {
            modules,
            identifier,
        }
    }
}
