use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Lookup {
    pub path: Expression,
    pub patterns: Vec<Expression>,
}

impl Lookup {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::Lookup) -> Self {
        let path = Expression::convert(analyzer, ast.path);
        let patterns = ast
            .patterns
            .into_iter()
            .map(|pat| Expression::convert_pattern(analyzer, pat))
            .collect();
        Self { path, patterns }
    }
}
