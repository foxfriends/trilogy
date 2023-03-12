use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::Module;
use trilogy_parser::syntax::Document;
use trilogy_parser::Spanned;

pub(crate) fn analyze_document(analyzer: &mut Analyzer, document: Document) -> Module {
    analyze_definitions(analyzer, document.span(), document.definitions)
}
