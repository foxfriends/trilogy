use super::*;
use crate::Analyzer;
use trilogy_parser::syntax;

#[derive(Clone, Debug)]
pub struct Handled {
    pub expression: Expression,
    pub handlers: Vec<Handler>,
}

impl Handled {
    pub(super) fn convert_block(analyzer: &mut Analyzer, ast: syntax::HandledBlock) -> Self {
        let expression = Expression::convert_block(analyzer, ast.block);
        let handlers = ast
            .handlers
            .into_iter()
            .map(|ast| Handler::convert(analyzer, ast))
            .collect();
        Self {
            expression,
            handlers,
        }
    }
}
