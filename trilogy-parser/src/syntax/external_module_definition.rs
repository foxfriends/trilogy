use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ExternalModuleDefinition {
    start: Token,
    pub name: Identifier,
    pub locator: StringLiteral,
}
