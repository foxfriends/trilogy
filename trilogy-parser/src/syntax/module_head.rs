use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleHead {
    pub name: Identifier,
    pub parameters: Option<ModuleParameters>,
}

#[derive(Clone, Debug)]
pub struct ModuleParameters {
    pub parameters: Vec<Identifier>,
    end: Token,
}
