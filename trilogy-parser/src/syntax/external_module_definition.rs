use super::*;

#[derive(Clone, Debug)]
pub struct ExternalModuleDefinition {
    pub name: Identifier,
    pub locator: StringLiteral,
}
