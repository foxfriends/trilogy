use super::*;
use crate::Analyzer;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Handled {
    pub expression: Expression,
    pub handlers: Vec<Handler>,
}

impl Handled {
    pub(super) fn convert_block(analyzer: &mut Analyzer, ast: syntax::HandledBlock) -> Expression {
        let span = ast.span();
        let expression = Expression::convert_block(analyzer, ast.block);
        let handlers = ast
            .handlers
            .into_iter()
            .map(|ast| Handler::convert(analyzer, ast))
            .collect();
        Expression::handled(
            span,
            Self {
                expression,
                handlers,
            },
        )
    }

    pub(super) fn convert_expression(
        analyzer: &mut Analyzer,
        ast: syntax::HandledExpression,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(analyzer, ast.expression);
        let handlers = ast
            .handlers
            .into_iter()
            .map(|ast| Handler::convert(analyzer, ast))
            .collect();
        Expression::handled(
            span,
            Self {
                expression,
                handlers,
            },
        )
    }
}
