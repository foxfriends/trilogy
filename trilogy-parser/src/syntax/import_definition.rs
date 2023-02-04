use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ImportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
    pub module: ModulePath,
}
