use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Code, Cond, Value};
use trilogy_parser::syntax::IfStatement;
use trilogy_parser::Spanned;

pub(super) fn analyze_if_statement(analyzer: &mut Analyzer, statement: IfStatement) -> Vec<Code> {
    let span = statement.span();
    let mut conds = vec![];

    for branch in statement.branches {
        let condition = vec![analyze_poetry(analyzer, branch.condition).into()];
        let body = vec![analyze_prose(analyzer, branch.body)];
        conds.push(Cond::new(condition, body));
    }

    if let Some(body) = statement.if_false {
        // TODO: a bit of a hack, should probably have kept the `else` token around for this
        let condition = vec![Value::from(true).at(body.span()).into()];
        conds.push(Cond::new(condition, vec![analyze_prose(analyzer, body)]));
    }

    vec![Value::cond(conds).at(span).into()]
}
