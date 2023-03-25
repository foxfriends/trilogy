use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Module {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub definitions: Definitions,
}

impl Module {
    pub(crate) fn convert(analyzer: &mut Analyzer, ast: syntax::Document) -> Self {
        let span = ast.span();
        let mut definitions = ast
            .definitions
            .iter()
            .filter_map(|ast| Definition::declare(analyzer, ast))
            .collect::<Definitions>();
        for definition in ast.definitions {
            Definition::convert_into(analyzer, definition, &mut definitions);
        }
        Self {
            span,
            parameters: vec![],
            definitions,
        }
    }
}
