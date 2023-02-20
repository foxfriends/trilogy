use super::*;
use crate::{
    format::{PrettyPrint, PrettyPrinted, PrettyPrinter},
    Parser, Spanned,
};
use pretty::DocAllocator;
use source_span::Span;
use trilogy_scanner::{Token, TokenType::*};

#[derive(Clone, Debug, PrettyPrintSExpr)]
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
            .expect("input should start with `StartOfFile`");

        if let Ok(token) = parser.expect(ByteOrderMark) {
            #[cfg(feature = "lax")]
            {
                parser.warn(SyntaxError::new(
                    token.span,
                    "the file contains a byte-order mark",
                ));
            }

            #[cfg(not(feature = "lax"))]
            {
                parser.error(SyntaxError::new(
                    token.span,
                    "the file contains a byte-order mark",
                ));
            }
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

        if !parser.is_line_start {
            #[cfg(feature = "lax")]
            parser.warn(SyntaxError::new_spanless(
                "the document does not end with a new-line character",
            ));

            #[cfg(not(feature = "lax"))]
            parser.error(SyntaxError::new_spanless(
                "no new line found at end of file",
            ));
        }

        let end = parser
            .expect(EndOfFile)
            .expect("input should end with `EndOfFile`");

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

#[cfg(test)]
mod test {
    use super::*;

    test_parse_whole!(document_empty: "" => Document::parse => "(Document () [])");
    test_parse_whole!(document_empty_newline: "\n" => Document::parse => "(Document () [])");

    test_parse_whole_error!(document_empty_bom: "\u{feff}" => Document::parse => "the file contains a byte-order mark");
    test_parse_whole_error!(document_empty_bom_newline: "\u{feff}\n" => Document::parse => "the file contains a byte-order mark");

    test_parse_whole!(document_documented: "#! hello\n#! world" => Document::parse => "(Document (Documentation) [])");
    test_parse_whole!(document_documented_with_def: "#! hello\n#! world\n\n## Hello\nfunc f x = x\n" => Document::parse => "(Document (Documentation) [(Definition (Documentation) _)])");

    test_parse_whole_error!(document_no_final_newline: "func f x = x" => Document::parse => "no new line found at end of file");

    test_parse_whole!(document_multiple_defs: "func f x = x\nfunc f y = y\nfunc g x = x\n" => Document::parse => "(Document () [(Definition () _) (Definition () _) (Definition () _)])");
    test_parse_whole_error!(document_defs_no_newline: "func f x = x func f y = y\n" => Document::parse => "definitions must be separated by line breaks");

    test_parse_whole!(document_module_empty: "module A {}\n" => Document::parse => "(Document () [(Definition () _)])");
    test_parse_whole!(document_module_nested: "module A {\n    module B { }\n}\n" => Document::parse => "(Document () [(Definition () _)])");

    test_parse_whole_error!(document_module_no_end_newline: "module A {\n    module B { }}\n" => Document::parse => "definition in module must end with a line break");
    test_parse_whole_error!(document_module_no_start_newline: "module A {module B { }}\n" => Document::parse => "definitions must be separated by line breaks");

    #[test]
    #[rustfmt::skip]
    fn document_error_recovery() {
        use crate::Parser;
        use trilogy_scanner::Scanner;

        let scanner = Scanner::new("func f = y\nfunc f x = x\n");
        let mut parser = Parser::new(scanner);
        let parse = Document::parse(&mut parser);
        assert_eq!(parse.definitions.len(), 1, "expected one definition to succeed");
        assert_eq!(parser.errors.len(), 1, "expected one definition to fail");
    }
}
