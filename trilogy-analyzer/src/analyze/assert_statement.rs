use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Code, Step, Violation};
use trilogy_parser::syntax::AssertStatement;
use trilogy_parser::Spanned;

pub(super) fn analyze_assert_statement(
    analyzer: &mut Analyzer,
    assertion: AssertStatement,
) -> Vec<Code> {
    let whole_span = assertion.span();
    let span = assertion.assertion.span();
    let condition = Step::evaluation(analyze_poetry(analyzer, assertion.assertion)).at(span);
    let violation = Step::violation(Violation::AssertionError(
        assertion
            .message
            .map(|message| analyze_poetry(analyzer, message))
            .unwrap(), // TODO: don't unwrap, and put in a default
    ))
    .at(whole_span);
    vec![Step::selection(condition, violation).at(span).into()]
}
