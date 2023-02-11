use super::*;
use crate::{
    format::{PrettyPrint, PrettyPrinted, PrettyPrinter},
    Parser, Spanned, TokenPattern,
};
use pretty::DocAllocator;
use source_span::Span;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
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

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Definition {
    pub documentation: Option<Documentation>,
    pub item: DefinitionItem,
}

impl Definition {
    fn parse_until(
        parser: &mut Parser,
        until_pattern: impl TokenPattern,
    ) -> SyntaxResult<Option<Self>> {
        let documentation = Documentation::parse_outer(parser);

        let token = parser.peek();
        if until_pattern.matches(token) {
            if let Some(documentation) = documentation {
                let error = SyntaxError::new(
                    documentation.span(),
                    "outer documentation comment must precede the item it documents",
                );
                parser.error(error.clone());
                return Err(error);
            } else {
                return Ok(None);
            }
        }

        let item = match token.token_type {
            KwModule => {
                let head = ModuleHead::parse(parser)?;
                let token = parser.peek();
                match token.token_type {
                    KwAt => DefinitionItem::ExternalModule(Box::new(
                        ExternalModuleDefinition::parse(parser, head)?,
                    )),
                    OBrace => {
                        DefinitionItem::Module(Box::new(ModuleDefinition::parse(parser, head)?))
                    }
                    _ => {
                        let error = SyntaxError::new(
                            token.span,
                            "expected `at` for an external module, or { for a local module",
                        );
                        parser.error(error.clone());
                        return Err(error);
                    }
                }
            }
            KwImport => {
                let start = parser.expect(KwImport).unwrap();

                // Module reference is conveniently a superset of identifier, which is the first
                // bit of a list of identifiers.
                //
                // Parse one of those, then check the next token to determine the way forward.
                let first = ModuleReference::parse(parser)?;
                if parser.check([KwAs, OpDot]).is_ok() {
                    DefinitionItem::ModuleImport(Box::new(ModuleImportDefinition::parse(
                        parser, start, first,
                    )?))
                } else {
                    DefinitionItem::Import(Box::new(ImportDefinition::parse(
                        parser,
                        start,
                        first.try_into()?,
                    )?))
                }
            }
            KwExport => DefinitionItem::Export(Box::new(ExportDefinition::parse(parser)?)),
            KwRule => DefinitionItem::Rule(Box::new(RuleDefinition::parse(parser)?)),
            KwProc => DefinitionItem::Procedure(Box::new(ProcedureDefinition::parse(parser)?)),
            KwFunc => DefinitionItem::Function(Box::new(FunctionDefinition::parse(parser)?)),
            KwTest => DefinitionItem::Test(Box::new(TestDefinition::parse(parser)?)),
            _ => {
                let error = SyntaxError::new(token.span, "unexpected token in module body");
                parser.error(error.clone());
                return Err(error);
            }
        };
        Ok(Some(Self {
            documentation,
            item,
        }))
    }

    pub(crate) fn parse_in_document(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        Self::parse_until(parser, EndOfFile)
    }

    pub(crate) fn parse_in_module(parser: &mut Parser) -> SyntaxResult<Option<Self>> {
        Self::parse_until(parser, [EndOfFile, CBrace])
    }
}

impl Spanned for Definition {
    fn span(&self) -> Span {
        match &self.documentation {
            Some(documentation) => documentation.span().union(self.item.span()),
            None => self.item.span(),
        }
    }
}

impl<'a> PrettyPrint<'a> for Definition {
    fn pretty_print(&self, printer: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        printer.nil()
    }
}
