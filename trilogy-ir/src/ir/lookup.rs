use super::*;
use crate::Converter;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Lookup {
    pub path: Expression,
    pub patterns: Vec<Expression>,
}

impl Lookup {
    pub(super) fn convert(converter: &mut Converter, ast: syntax::Lookup) -> Self {
        let path = Expression::convert(converter, ast.path);
        let patterns = ast
            .patterns
            .into_iter()
            .map(|pat| Expression::convert_pattern(converter, pat))
            .collect();
        Self { path, patterns }
    }
}
