use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// An export declaration item.
///
/// ```trilogy
/// export something, something_else
/// ```
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct ExportDefinition {
    pub export: Token,
    pub names: Vec<Identifier>,
}

impl Spanned for ExportDefinition {
    fn span(&self) -> Span {
        if self.names.is_empty() {
            self.export.span
        } else {
            self.export.span.union(self.names.span())
        }
    }
}

impl ExportDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let export = parser
            .expect(TokenType::KwExport)
            .expect("Caller should find `export` keyword.");
        let mut names = vec![];
        while {
            names.push(Identifier::parse(parser)?);
            parser.expect(TokenType::OpComma).is_ok()
        } {}
        Ok(Self { export, names })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(export_single: "export x" => ExportDefinition::parse => "(ExportDefinition _ [_])");
    test_parse!(export_many: "export x, y, z" => ExportDefinition::parse => "(ExportDefinition _ [_ _ _])");
    test_parse_error!(export_not_ident: "export x y" => ExportDefinition::parse);
    test_parse_error!(export_none: "export" => ExportDefinition::parse);
}
