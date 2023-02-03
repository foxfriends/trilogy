use super::*;

#[derive(Clone, Debug)]
pub struct ImportDefinition {
    pub names: Vec<Identifier>,
    pub module: ModulePath,
}
