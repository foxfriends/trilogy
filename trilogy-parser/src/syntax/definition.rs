use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug)]
pub enum DefinitionItem {
    Module(Box<ModuleDefinition>),
    ExternalModule(Box<ExternalModuleDefinition>),
    Procedure(Box<ProcedureDefinition>),
    Function(Box<FunctionDefinition>),
    Rule(Box<RuleDefinition>),
    Import(Box<ImportDefinition>),
    ModuleImport(Box<ModuleImportDefinition>),
    Export(Box<ExportDefinition>),
    Test(Box<TestDefinition>),
    SyntaxError(Box<SyntaxError>),
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub documentation: Option<Documentation>,
    pub item: DefinitionItem,
}

impl Definition {
    pub(crate) fn syntax_error(error: SyntaxError) -> Self {
        Self {
            documentation: None,
            item: DefinitionItem::SyntaxError(Box::new(error)),
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> Option<Self> {
        if parser.check(EndOfFile).is_some() {
            return None;
        }
        todo!()
    }
}
