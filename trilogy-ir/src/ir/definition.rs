use super::*;
use source_span::Span;

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Test(Box<TestDefinition>),
    Alias(Box<Alias>),
    Module(Box<Module>),
}

#[derive(Clone, Debug)]
pub struct Definition {
    span: Span,
    pub name: Identifier,
    pub item: DefinitionItem,
    pub is_exported: bool,
}
