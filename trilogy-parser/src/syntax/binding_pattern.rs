use super::*;

#[derive(Clone, Debug)]
pub struct BindingPattern {
    pub mutable: MutModifier,
    pub identifier: Identifier,
}
