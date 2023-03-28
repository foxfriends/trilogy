use super::*;
use crate::Analyzer;
use source_span::Span;
use trilogy_parser::{syntax, Spanned};

#[derive(Clone, Debug)]
pub struct Procedure {
    span: Span,
    pub parameters: Vec<Pattern>,
    pub body: Expression,
}

impl Procedure {
    pub(super) fn convert(analyzer: &mut Analyzer, ast: syntax::ProcedureDefinition) -> Self {
        let span = ast.span();
        let parameters = ast
            .head
            .parameters
            .into_iter()
            .map(|param| Pattern::convert(analyzer, param))
            .collect();
        let body = Expression::convert_block(analyzer, ast.body);
        Self {
            span,
            parameters,
            body,
        }
    }
}
