use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType};

/// An export declaration item.
///
/// ```trilogy
/// export something, something_else
/// ```
#[derive(Clone, Debug)]
pub struct ExportDefinition {
    pub export: Token,
    pub names: Vec<Identifier>,
    pub span: Span,
}

impl Spanned for ExportDefinition {
    fn span(&self) -> Span {
        self.span
    }
}

impl ExportDefinition {
    pub(crate) fn parse(parser: &mut Parser) -> SyntaxResult<Self> {
        let export = parser.expect(TokenType::KwExport).unwrap();
        let mut names = vec![];
        while {
            names.push(Identifier::parse(parser)?);
            parser.expect(TokenType::OpComma).is_ok()
        } {}

        let span = match names.last() {
            None => export.span,
            Some(name) => export.span.union(name.span()),
        };

        Ok(Self {
            export,
            names,
            span,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(export_single: "export x" => ExportDefinition::parse => ExportDefinition { names: [_], .. });
    test_parse!(export_many: "export x, y, z" => ExportDefinition::parse => ExportDefinition { names: [_, _, _], .. });
    test_parse_error!(export_not_ident: "export x y" => ExportDefinition::parse);
    test_parse_error!(export_none: "export" => ExportDefinition::parse);
}
