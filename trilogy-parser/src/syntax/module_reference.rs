use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ModuleReference {
    pub name: Identifier,
    pub arguments: Option<Vec<ModuleReference>>,
    end: Token,
}
