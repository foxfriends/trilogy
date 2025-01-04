use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

/// The pre- and post-amble of the Trilogy file.
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub(crate) struct Amble {
    pub start_of_file: Token,
    pub content: Document,
    pub end_of_file: Token,
}

impl Spanned for Amble {
    fn span(&self) -> Span {
        self.start_of_file.span.union(self.end_of_file.span)
    }
}

impl Amble {
    pub(crate) fn parse(parser: &mut Parser) -> Self {
        let start_of_file = parser
            .expect(StartOfFile)
            .expect("input should start with `StartOfFile`");

        if let Ok(token) = parser.expect(ByteOrderMark) {
            #[cfg(feature = "lax")]
            parser.warn(SyntaxError::new(
                token.span,
                "the file contains a byte-order mark",
            ));

            #[cfg(not(feature = "lax"))]
            parser.error(SyntaxError::new(
                token.span,
                "the file contains a byte-order mark",
            ));
        }

        let content = Document::parse(parser);

        let end_of_file = parser
            .expect(EndOfFile)
            .expect("input should end with `EndOfFile`");

        Self {
            start_of_file,
            content,
            end_of_file,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse_whole!(amble_empty: "" => Amble::parse => "(Amble _ (Document () []) _)");
    test_parse_whole!(amble_empty_newline: "\n" => Amble::parse => "(Amble _ (Document () []) _)");
    test_parse_whole_error!(amble_empty_bom: "\u{feff}" => Amble::parse => "the file contains a byte-order mark");
    test_parse_whole_error!(amble_empty_bom_newline: "\u{feff}\n" => Amble::parse => "the file contains a byte-order mark");
}
