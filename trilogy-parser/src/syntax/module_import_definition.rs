use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleImportDefinition {
    start: Token,
    pub module: ModulePath,
    pub name: Identifier,
}
