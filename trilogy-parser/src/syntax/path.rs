use super::*;

#[derive(Clone, Debug)]
pub struct Path {
    pub modules: Vec<ModuleReference>,
    pub identifier: Identifier,
}
