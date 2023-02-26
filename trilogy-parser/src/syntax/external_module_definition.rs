use super::*;
use crate::{Parser, Spanned};
use trilogy_scanner::TokenType;

#[derive(Clone, Debug, Spanned, PrettyPrintSExpr)]
pub struct ExternalModuleDefinition {
    pub head: ModuleHead,
    pub locator: StringLiteral,
}

impl ExternalModuleDefinition {
    pub(crate) fn parse(parser: &mut Parser, head: ModuleHead) -> SyntaxResult<Self> {
        parser
            .expect(TokenType::KwAt)
            .expect("Caller should find `at` keyword.");
        let locator = StringLiteral::parse(parser)?;
        let module = Self { head, locator };
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
            (StringLiteral))))");

    test_parse_error!(moduleat_not_string: "module Hello at 3" => Definition::parse_in_document);
    test_parse_error!(moduleat_args_invalid: "module Hello x y at \"./here.tri\"" => Definition::parse_in_document);
}
