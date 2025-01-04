use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// An external module definition item.
///
/// ```trilogy
/// module external_module at "./some/path.tri" use imported_ident
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ExternalModuleDefinition {
    /// The module declaration.
    pub head: ModuleHead,
    /// The `at` token
    pub at: Token,
    /// The string representing the path at which to find the module.
    pub locator: StringLiteral,
    pub module_use: Option<ModuleUse>,
    span: Span,
}

impl Spanned for ExternalModuleDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ExternalModuleDefinition {
    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        let at = parser
            .expect(TokenType::KwAt)
            .expect("Caller should find `at` keyword.");
        let locator = StringLiteral::parse(parser)?;
        let module_use = if parser.check(TokenType::KwUse).is_ok() {
            Some(ModuleUse::parse(parser)?)
        } else {
            None
        };

        let span = match &module_use {
            Some(uses) => head.span().union(uses.span()),
            None => head.span().union(locator.span()),
        };

        let module = Self {
            span,
            head,
            at,
            locator,
            module_use,
        };
        if !module.head.parameters.is_empty() {
            parser.error(SyntaxError::new(
                module.span(),
                "external module may not take parameters",
            ));
        }
        Ok(module)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(moduleat_ok: "module Hello at \"./here.tri\"" => Definition::parse_in_document => "
      (Definition
        _
        (DefinitionItem::ExternalModule
          (ExternalModuleDefinition
            (ModuleHead _ [])
            _
            (StringLiteral)
            _)))");

    test_parse!(moduleat_use: "module Hello at \"./here.tri\" use hello, world" => Definition::parse_in_document => "
      (Definition
        _
        (DefinitionItem::ExternalModule
          (ExternalModuleDefinition
            (ModuleHead _ [])
            _
            (StringLiteral)
            (ModuleUse _ _))))");

    test_parse_error!(moduleat_not_string: "module Hello at 3" => Definition::parse_in_document);
    test_parse_error!(moduleat_args_invalid: "module Hello x y at \"./here.tri\"" => Definition::parse_in_document);
}
