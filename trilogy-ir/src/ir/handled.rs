use super::*;
use crate::Converter;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Handled {
    pub expression: Expression,
    pub handlers: Vec<Handler>,
}

impl Handled {
    pub(super) fn convert_block(
        converter: &mut Converter,
        ast: syntax::HandledBlock,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert_block(converter, ast.block);
        let handlers = ast
            .handlers
            .into_iter()
            .map(|ast| Handler::convert_blocks(converter, ast))
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
        converter: &mut Converter,
        ast: syntax::HandledExpression,
    ) -> Expression {
        let span = ast.span();
        let expression = Expression::convert(converter, ast.expression);
        let handlers = ast
            .handlers
            .into_iter()
            .map(|ast| Handler::convert_expressions(converter, ast))
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
