use super::*;
use trilogy_scanner::Token;

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    start: Token,
    pub head: ProcedureHead,
    pub body: Block,
}
