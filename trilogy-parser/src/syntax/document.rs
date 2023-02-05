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
                "The file contains a byte-order mark.".to_owned(),
            ));
        }
    }

    pub(crate) fn parse(parser: &mut Parser) -> Self {
        Document::preamble(parser);

        // Special case for the empty file rule
        if parser.expect(EndOfFile).is_some() {
            return Self {
                documentation: None,
                definitions: vec![],
            };
        }

        let documentation = Documentation::parse_inner(parser);

        let mut definitions = vec![];
        while let Some(definition) = Definition::parse(parser) {
            definitions.push(definition);
        }

        if parser.expect(EndOfLine).is_none() {
            let syntax_error = SyntaxError::new_spanless(
                "The document does not end with a new-line character.".to_owned(),
            );

            #[cfg(feature = "lax")]
            parser.warn(syntax_error);

            #[cfg(not(feature = "lax"))]
            definitions.push(Definition::syntax_error(parser.error(syntax_error)));
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
