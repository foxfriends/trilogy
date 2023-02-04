use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleDefinition {
    start: Token,
    pub head: ModuleHead,
    pub body: Vec<Definition>,
    end: Token,
}
