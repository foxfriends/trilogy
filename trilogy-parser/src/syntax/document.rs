use super::*;
use crate::Parser;
use trilogy_scanner::TokenType::*;

#[derive(Clone, Debug)]
pub struct Document {
    pub documentation: Option<Documentation>,
    pub definitions: Vec<Definition>,
}

impl Document {
    fn preamble(parser: &mut Parser) {
        parser
            .expect(StartOfFile)
            .expect("The file better have a beginning... otherwise something is very wrong.");

        #[cfg(feature = "lax")]
        if let Some(token) = parser.expect(ByteOrderMark) {
            parser.warn(SyntaxError::new(
                vec![token],
                "The file contains a byte-order mark.",
            ));
        }
    }

    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwImport, KwExport, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser) -> Self {
        Document::preamble(parser);

        // Special case for the empty file rule
        if parser.expect(EndOfFile).is_ok() {
            return Self {
                documentation: None,
                definitions: vec![],
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
            if parser.check(EndOfFile).is_some() {
                break;
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

        parser
            .expect(EndOfFile)
            .expect("The file better have an end... otherwise something is very wrong.");

        Self {
            documentation,
            definitions,
        }
    }
}
