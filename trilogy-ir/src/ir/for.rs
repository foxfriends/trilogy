use super::*;
use crate::Analyzer;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct For {
    pub iterator: Query,
    pub body: Expression, // TODO: is a for loop just a comprehension into nothing?
}

impl For {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::ForStatement) -> Expression {
        let _span = ast.span();
        let _else_block = ast
            .else_block
            .map(|ast| Expression::convert_block(analyzer, ast));

        let _ = ast
            .branches
            .into_iter()
            .map(|branch| {
                let iterator = Query::convert(analyzer, branch.query);
                let body = Expression::convert_block(analyzer, branch.body);
                For { iterator, body }
            })
            .collect::<Vec<_>>();
        todo!("too much to do this morning, will have to revisit")
    }
}
