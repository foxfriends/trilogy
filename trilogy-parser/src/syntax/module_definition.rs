use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    pub parameters: Option<Vec<Identifier>>,
    pub definitions: Vec<Definition>,
    end: Token,
}
