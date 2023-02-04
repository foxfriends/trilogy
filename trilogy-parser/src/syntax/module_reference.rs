use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleReference {
    pub name: Identifier,
    pub arguments: Option<ModuleArguments>,
}

#[derive(Clone, Debug)]
pub struct ModuleArguments {
    pub arguments: Vec<ModuleReference>,
    end: Token,
}
