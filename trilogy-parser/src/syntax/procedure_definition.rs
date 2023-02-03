use super::*;

#[derive(Clone, Debug)]
pub struct ProcedureDefinition {
    pub head: ProcedureHead,
    pub body: Block,
}
