use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Assignment, Code, Step, Violation};
use trilogy_parser::syntax::{AssignmentStrategy, Statement};
use trilogy_parser::Spanned;

pub(super) fn analyze_statement(analyzer: &mut Analyzer, statement: Statement) -> Vec<Code> {
    match statement {
        Statement::Assert(assertion) => analyze_assert_statement(analyzer, *assertion),
        Statement::Block(block) => vec![analyze_prose(analyzer, *block)],
        Statement::Expression(expression) => vec![analyze_poetry(analyzer, *expression).into()],
        Statement::Assignment(assignment)
            if matches!(assignment.strategy, AssignmentStrategy::Direct(..)) =>
        {
            vec![Assignment::new(
                assignment.span(),
                analyze_lvalue(analyzer, assignment.lhs),
                analyze_poetry(analyzer, assignment.rhs),
            )
            .into()]
        }
        Statement::End(end_statement) => {
            vec![Step::Contradiction.at(end_statement.span()).into()]
        }
        Statement::Exit(exit_statement) => {
            let span = exit_statement.span();
            vec![Step::violation(Violation::Exit(analyze_poetry(
                analyzer,
                exit_statement.expression,
            )))
            .at(span)
            .into()]
        }
        Statement::FunctionAssignment(assignment) => {
            analyze_function_assignment(analyzer, *assignment)
        }
        _ => todo!(),
    }
}
