use super::*;
use source_span::Span;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Module {
    pub span: Span,
    pub imported_modules: HashMap<Id, Evaluation>,
    pub imported_items: HashMap<Id, Evaluation>,
    pub items: HashMap<ItemKey, Vec<Item>>,
    pub tests: Vec<Test>,
    pub exported_items: HashMap<String, Export>,
}
