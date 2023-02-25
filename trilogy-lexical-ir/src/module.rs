use super::*;
use source_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub renames: Vec<Rename>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: Vec<String>,
}
