use super::*;

#[derive(Clone, Debug)]
pub struct ModuleImportDefinition {
    pub module: ModulePath,
    pub name: Identifier,
}
