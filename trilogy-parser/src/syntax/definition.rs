use super::*;
use crate::{Parser, Spanned};
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
}

#[derive(Clone, Debug)]
pub struct Definition {
    pub documentation: Option<Documentation>,
    pub item: DefinitionItem,
}

impl Definition {
    pub(crate) fn parse(parser: &mut Parser) -> Option<Self> {
        loop {
            let documentation = Documentation::parse_outer(parser);
            match parser.peek().token_type {
                EndOfFile => {
                    // As a fun coincidence, the end of file is only
                    // an error if there was documentation. Otherwise,
                    // it's just the end of the file, no more definitions.
                    parser.error(SyntaxError::new(
                        documentation?.span(),
                        "Documentation must be accompanied by the item that it documents."
                            .to_owned(),
                    ));
                }
                KwModule => {}
                _ => {
                    // Any unexpected character we can just start collecting to report
                    // all in one big chunk when a valid definition item token (or end
                    // of file) is found.
                    //
                    // Documentation before such invalid characters is discarded, likely
                    // the error is caused by a missing comment marker or a typo in one
                    // of the definition item tokens, so the documentation would have
                    // applied successfully.
                    parser.discard();
                    continue;
                }
            }
            break None;
        }
    }
}
