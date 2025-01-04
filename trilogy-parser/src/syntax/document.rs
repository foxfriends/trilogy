use super::*;
use crate::{Parser, Spanned};
use source_span::Span;
use trilogy_scanner::TokenType::*;

/// A complete Trilogy document.
///
/// Similar to a module, but without a module declaration.
#[derive(Clone, Debug, PrettyPrintSExpr)]
pub struct Document {
    /// The inner documentation of this document, intended to document the whole file.
    pub documentation: Option<Documentation>,
    /// The definitions contained within this file.
    pub definitions: Vec<Definition>,
}

impl Spanned for Document {
    fn span(&self) -> Span {
        match (&self.documentation, self.definitions.last()) {
            (Some(doc), Some(def)) => doc.span().union(def.span()),
            (Some(doc), None) => doc.span(),
            (None, Some(def)) => self.definitions.first().unwrap().span().union(def.span()),
            (None, None) => Span::default(),
        }
    }
}

impl Document {
    fn synchronize(parser: &mut Parser) {
        parser.synchronize([
            DocOuter, KwModule, KwFunc, KwProc, KwRule, KwConst, KwExport, EndOfFile,
        ]);
    }

    pub(crate) fn parse(parser: &mut Parser) -> Self {
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

        Self {
            documentation,
            definitions,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_parse!(document_empty: "" |parser| Document::parse(&mut parser) => "(Document () [])");
    test_parse!(document_empty_newline: "\n" |parser| Document::parse(&mut parser) => "(Document () [])");

    test_parse!(document_documented: "#! hello\n#! world" |parser| Document::parse(&mut parser) => "(Document (Documentation _) [])");
    test_parse!(document_documented_with_def: "#! hello\n#! world\n\n## Hello\nfunc f x = x\n" |parser| Document::parse(&mut parser) => "(Document (Documentation _) [(Definition (Documentation _) _)])");

    test_parse_error!(document_no_final_newline: "func f x = x" |parser| Document::parse(&mut parser) => "no new line found at end of file");

    test_parse!(document_multiple_defs: "func f x = x\nfunc f y = y\nfunc g x = x\n" |parser| Document::parse(&mut parser) => "(Document () [(Definition () _) (Definition () _) (Definition () _)])");
    test_parse_error!(document_defs_no_newline: "func f x = x func f y = y\n" |parser| Document::parse(&mut parser) => "definitions must be separated by line breaks");

    test_parse!(document_module_empty: "module A {}\n" |parser| Document::parse(&mut parser) => "(Document () [(Definition () _)])");
    test_parse!(document_module_nested: "module A {\n    module B { }\n}\n" |parser| Document::parse(&mut parser) => "(Document () [(Definition () _)])");

    test_parse_error!(document_module_no_end_newline: "module A {\n    module B { }}\n" |parser| Document::parse(&mut parser) => "definition in module must end with a line break");
    test_parse_error!(document_module_no_start_newline: "module A {module B { }}\n" |parser| Document::parse(&mut parser) => "definitions must be separated by line breaks");

    #[test]
    #[rustfmt::skip]
    fn document_error_recovery() {
        use crate::Parser;
        use trilogy_scanner::Scanner;
        let scanner = Scanner::new("func f = y\nfunc f x = x\n");
        let mut parser = Parser::new(scanner);
        let Amble { content, .. } = Amble::parse(&mut parser);
        assert_eq!(content.definitions.len(), 1, "expected one definition to succeed");
        assert_eq!(parser.errors.len(), 1, "expected one definition to fail");
    }
}
