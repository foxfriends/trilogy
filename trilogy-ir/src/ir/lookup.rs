use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Lookup {
    pub path: Expression,
    pub patterns: Vec<Pattern>,
}

impl Lookup {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::Lookup) -> Self {
        let path = Expression::convert_path(analyzer, ast.path);
        let patterns = ast
            .patterns
            .into_iter()
            .map(|pat| Pattern::convert(analyzer, pat))
            .collect();
        Self { path, patterns }
    }
}
