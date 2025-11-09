use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// An external module definition item.
///
/// ```trilogy
/// import "./some/path.tri" as name use imported_ident
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ImportDefinition {
    /// The `import` token.
    pub import: Token,
    /// The module locator.
    pub locator: StringLiteral,
    pub type_as: Option<TypeAs>,
    pub type_use: Option<TypeUse>,
    span: Span,
}

impl Spanned for ImportDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct TypeAs {
    /// The `as` token.
    pub r#as: Token,
    /// The identifier.
    pub identifier: Identifier,
    span: Span,
}

impl TypeAs {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let r#as = parser.expect(TokenType::KwAs).unwrap();
        let identifier = Identifier::parse(parser)?;
        let span = r#as.span.union(identifier.span());
        Ok(Self {
            r#as,
            identifier,
            span,
        })
    }
}

impl ImportDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let import = parser.expect(TokenType::KwImport).unwrap();

        // TODO: support computed module locators (constant expression)
        let locator = StringLiteral::parse(parser)?;

        let type_as = if parser.check(TokenType::KwAs).is_ok() {
            Some(TypeAs::parse(parser)?)
        } else {
            None
        };

        let type_use = if parser.check(TokenType::KwUse).is_ok() {
            Some(TypeUse::parse(parser)?)
        } else {
            None
        };

        let span = match &type_use {
            Some(uses) => import.span.union(uses.span()),
            None => match &type_as {
                Some(type_as) => import.span.union(type_as.span),
                None => import.span.union(locator.span()),
            },
        };

        let module = Self {
            import,
            locator,
            type_as,
            type_use,
            span,
        };
        Ok(module)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(import_ok: "import \"./here.tri\" as hello" => Definition::parse_in_document => "
      (Definition
        _
        (DefinitionItem::Import
          (ImportDefinition
            _
            (StringLiteral)
            (TypeAs _ _)
            ())))");

    test_parse!(import_use: "import \"./here.tri\" use hello, world" => Definition::parse_in_document => "
      (Definition
        _
        (DefinitionItem::Import
          (ImportDefinition
            _
            (StringLiteral)
            ()
            (TypeUse _ _))))");

    test_parse_error!(import_not_string: "import Hello as hello" => Definition::parse_in_document);
    test_parse_error!(import_args_invalid: "import Hello x y as hello" => Definition::parse_in_document);
}
