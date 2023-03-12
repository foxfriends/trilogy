use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Code, Scope};
use trilogy_parser::syntax::Block;
use trilogy_parser::Spanned;

pub(super) fn analyze_prose(analyzer: &mut Analyzer, body: Block) -> Code {
    let mut scope = Scope {
        span: body.span(),
        code: vec![],
        handler: vec![],
    };
    for statement in body.statements {
        scope.code.extend(analyze_statement(analyzer, statement));
    }
    Code::Scope(Box::new(scope))
}
