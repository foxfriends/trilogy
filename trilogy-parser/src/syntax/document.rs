use super::*;
use crate::{
    format::{PrettyPrint, PrettyPrinted, PrettyPrinter},
    Parser, Spanned,
};
use pretty::DocAllocator;
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug)]
pub struct Document {
    start: Token,
    pub documentation: Option<Documentation>,
    pub definitions: Vec<Definition>,
    end: Token,
}

impl Spanned for Document {
    fn span(&self) -> Span {
        self.start.span.union(self.end.span())
    }
}

impl Document {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwImport, KwExport, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser) -> Self {
        let start = parser
            .expect(StartOfFile)
            .expect("The file better have a beginning... otherwise something is very wrong.");

        #[cfg(feature = "lax")]
        if let Some(token) = parser.expect(ByteOrderMark) {
            parser.warn(SyntaxError::new(
                vec![token],
                "The file contains a byte-order mark.",
            ));
        }

        // Special case for the empty file rule
        if let Ok(end) = parser.expect(EndOfFile) {
            return Self {
                start,
                documentation: None,
                definitions: vec![],
                end,
            };
        }

        let documentation = Documentation::parse_inner(parser);

        let mut definitions = vec![];
        loop {
            match Definition::parse_in_document(parser) {
                Ok(Some(definition)) => definitions.push(definition),
                Ok(None) => break,
                Err(..) => Document::synchronize(parser),
            }
        }

        if !parser.is_line_start() {
            #[cfg(feature = "lax")]
            parser.warn(SyntaxError::new_spanless(
                "The document does not end with a new-line character.",
            ));

            #[cfg(not(feature = "lax"))]
            parser.error(SyntaxError::new_spanless(
                "No new line found at end of file.",
            ));
        }

        let end = parser
            .expect(EndOfFile)
            .expect("The file better have an end... otherwise something is very wrong.");

        Self {
            start,
            documentation,
            definitions,
            end,
        }
    }
}

impl<'a> PrettyPrint<'a> for Document {
    fn pretty_print(&self, allocator: &'a PrettyPrinter) -> PrettyPrinted<'a> {
        let docs = self
            .documentation
            .iter()
            .map(|doc| doc.pretty_print(allocator))
            .chain(
                self.definitions
                    .iter()
                    .map(|def| def.pretty_print(allocator)),
            );
        allocator
            .intersperse(docs, allocator.hardline().append(allocator.hardline()))
            .append(allocator.hardline())
    }
}
