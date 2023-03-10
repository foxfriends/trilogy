use super::*;
use crate::Analyzer;
use source_span::Span;
use std::collections::HashMap;
use trilogy_parser::syntax::Document;
use trilogy_parser::Spanned;

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub renames: Vec<Rename>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: Vec<String>,
}

impl Module {
    pub(crate) fn analyze(_analyzer: &mut Analyzer, ast: Document) -> Self {
        let renames = vec![];
        let items = HashMap::new();
        let tests = vec![];
        let exported_items = vec![];

        Self {
            span: ast.span(),
            renames,
            items,
            tests,
            exported_items,
        }
    }
}
