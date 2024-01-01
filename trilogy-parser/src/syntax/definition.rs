use super::*;
use crate::{Parser, Spanned, TokenPattern};
use source_span::Span;
use trilogy_scanner::TokenType::*;

/// The various items that can be defined in a Trilogy module.
#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub enum DefinitionItem {
    /// An inline module definition.
    Module(Box<ModuleDefinition>),
    /// An external (imported) module definition.
    ExternalModule(Box<ExternalModuleDefinition>),
    /// A procedure definition.
    Procedure(Box<ProcedureDefinition>),
    /// A constant definition.
    Constant(Box<ConstantDefinition>),
    /// A function definition.
    Function(Box<FunctionDefinition>),
    /// A rule definition.
    Rule(Box<RuleDefinition>),
    /// An export declaration.
    Export(Box<ExportDefinition>),
    /// A test definition.
    Test(Box<TestDefinition>),
}

/// A definition in a Trilogy program.
///
/// ```trilogy
/// ## Documentation
/// proc definition!() {}
/// ```
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

        parser.peek();
        let is_line_start = parser.is_line_start;
        if until_pattern.matches(parser.peek()) {
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

        if !is_line_start {
            let error = SyntaxError::new(
                parser.peek().span,
                "definitions must be separated by line breaks",
            );
            parser.error(error);
        }

        let token = parser.peek();
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
                            "expected `at` for an external module, or `{` for a local module",
                        );
                        parser.error(error.clone());
                        return Err(error);
                    }
                }
            }
            KwExport => DefinitionItem::Export(Box::new(ExportDefinition::parse(parser)?)),
            KwConst => DefinitionItem::Constant(Box::new(ConstantDefinition::parse(parser)?)),
            KwRule => DefinitionItem::Rule(Box::new(RuleDefinition::parse(parser)?)),
            KwProc => DefinitionItem::Procedure(Box::new(ProcedureDefinition::parse(parser)?)),
            KwFunc => DefinitionItem::Function(Box::new(FunctionDefinition::parse(parser)?)),
            KwTest => DefinitionItem::Test(Box::new(TestDefinition::parse(parser)?)),
            DocInner => {
                let error = SyntaxError::new(
                    token.span,
                    "inner documentation is only supported at the top of a document",
                );
                parser.error(error.clone());
                return Err(error);
            }
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

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(def_proc: "proc hello!() {}" => Definition::parse_in_document => "(Definition () (DefinitionItem::Procedure _))");
    test_parse!(def_proc_in_module: "proc hello!() {}" => Definition::parse_in_module => "(Definition () (DefinitionItem::Procedure _))");
    test_parse!(def_func: "func hello x = x" => Definition::parse_in_document => "(Definition () (DefinitionItem::Function _))");
    test_parse!(def_func_in_module: "func hello x = x" => Definition::parse_in_module => "(Definition () (DefinitionItem::Function _))");
    test_parse!(def_fact: "rule hello(a, b)" => Definition::parse_in_document => "(Definition () (DefinitionItem::Rule _))");
    test_parse!(def_rule: "rule hello(a, b) <- x(a) and y(b)" => Definition::parse_in_document => "(Definition () (DefinitionItem::Rule _))");
    test_parse!(def_fact_in_module: "rule hello(a, b)" => Definition::parse_in_module => "(Definition () (DefinitionItem::Rule _))");
    test_parse!(def_rule_in_module: "rule hello(a, b) <- x(a) and y(b)" => Definition::parse_in_module => "(Definition () (DefinitionItem::Rule _))");
    test_parse!(def_module: "module X {}" => Definition::parse_in_document => "(Definition () (DefinitionItem::Module _))");
    test_parse!(def_module_in_module: "module X {}" => Definition::parse_in_module => "(Definition () (DefinitionItem::Module _))");
    test_parse!(def_external_module: "module X at \"./hello.tri\"" => Definition::parse_in_document => "(Definition () (DefinitionItem::ExternalModule _))");
    test_parse!(def_external_module_in_module: "module X at \"./hello.tri\"" => Definition::parse_in_module => "(Definition () (DefinitionItem::ExternalModule _))");
    test_parse_error!(def_module_invalid: "module X" => Definition::parse_in_document => "expected `at` for an external module, or `{` for a local module");
    test_parse_error!(def_module_invalid_in_module: "module X" => Definition::parse_in_module => "expected `at` for an external module, or `{` for a local module");
    test_parse!(def_export: "export a, b, c" => Definition::parse_in_document => "(Definition () (DefinitionItem::Export _))");
    test_parse!(def_export_in_module: "export a, b, c" => Definition::parse_in_module => "(Definition () (DefinitionItem::Export _))");
    test_parse!(def_test: "test \"hello\" {}" => Definition::parse_in_document => "(Definition () (DefinitionItem::Test _))");
    test_parse!(def_test_in_module: "test \"hello\" {}" => Definition::parse_in_module => "(Definition () (DefinitionItem::Test _))");
    test_parse!(def_documented: "## Hello this is a module\nmodule A {}" => Definition::parse_in_document => "(Definition (Documentation _) (DefinitionItem::Module _))");
    test_parse!(def_documented_in_module: "## Hello this is a module\nmodule A {}" => Definition::parse_in_module => "(Definition (Documentation _) (DefinitionItem::Module _))");
    test_parse!(def_nothing: "" => Definition::parse_in_document => "()");
    test_parse!(def_nothing_in_module: "" => Definition::parse_in_module => "()");
    test_parse_error!(def_documented_nothing: "## Hello this is a doc for nothing" => Definition::parse_in_document => "outer documentation comment must precede the item it documents");
    test_parse_error!(def_documented_nothing_in_module: "## Hello this is a doc for nothing" => Definition::parse_in_module => "outer documentation comment must precede the item it documents");
    test_parse_error!(def_documented_inner: "#! Hello this is a module\nmodule A {}" => Definition::parse_in_document => "inner documentation is only supported at the top of a document");
    test_parse_error!(def_documented_inner_in_module: "#! Hello this is a module\nmodule A {}" => Definition::parse_in_module => "inner documentation is only supported at the top of a document");
    test_parse_error!(def_no_keyword: "hello x = y" => Definition::parse_in_document => "unexpected token in module body");
    test_parse_error!(def_no_keyword_in_module: "hello x = y" => Definition::parse_in_module => "unexpected token in module body");
}
