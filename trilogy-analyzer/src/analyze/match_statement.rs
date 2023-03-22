use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Code, Collect, Cond, Reference, Step, Value};
use trilogy_parser::syntax::MatchStatement;
use trilogy_parser::Spanned;

pub(super) fn analyze_match_statement(
    analyzer: &mut Analyzer,
    statement: MatchStatement,
) -> Vec<Code> {
    let span = statement.span();

    let mut steps = vec![];
    let mut conds = vec![];

    let discriminant_span = statement.expression.span();
    let discriminant = Reference::temp(discriminant_span);
    steps.push(
        Step::unification(
            Value::declaration(discriminant.clone()).at(discriminant_span),
            analyze_poetry(analyzer, statement.expression),
        )
        .at(discriminant.span)
        .into(),
    );
    let discriminant_ref = Value::dereference(discriminant).at(discriminant_span);
    for case in statement.cases {
        let case_span = case.span();
        let mut condition: Vec<Code> = vec![];
        if let Some(pattern) = case.pattern {
            let pattern_span = pattern.span();
            let pattern = analyze_pattern(analyzer, pattern);
            // NOTE: this might be a weird one, as the only place where declarations
            // are persisting past the end of a collect... Might need a smarter
            // solution here.
            condition.push(
                Value::collect(Collect::new_scalar(
                    Step::unification(pattern, discriminant_ref.clone()).at(pattern_span),
                ))
                .at(pattern_span)
                .into(),
            );
        }
        match case.guard {
            None => {
                condition.push(Value::from(true).at(case_span).into());
            }
            Some(guard) => {
                condition.push(analyze_poetry(analyzer, guard).into());
            }
        }
        let body = vec![analyze_prose(analyzer, case.body)];
        conds.push(Cond::new(condition, body));
    }

    if let Some(body) = statement.else_case {
        // TODO: a bit of a hack, should probably have kept the `else` token around for this
        let condition = vec![Value::from(true).at(body.span()).into()];
        conds.push(Cond::new(condition, vec![analyze_prose(analyzer, body)]));
    }
    steps.push(Value::cond(conds).at(span).into());
    steps
}
