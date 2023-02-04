use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ExportDefinition {
    start: Token,
    pub names: Vec<Identifier>,
}
