use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Assignment, Code, Step, Violation};
use trilogy_parser::syntax::{AssignmentStrategy, Statement};
use trilogy_parser::Spanned;

pub(super) fn analyze_statement(analyzer: &mut Analyzer, statement: Statement) -> Vec<Code> {
    match statement {
        Statement::Assert(assertion) => analyze_assert_statement(analyzer, *assertion),
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
        Statement::Assignment(_assignment) => todo!(),
        Statement::Block(block) => vec![analyze_prose(analyzer, *block)],
        Statement::Break(..) => todo!(),
        Statement::Cancel(..) => todo!(),
        Statement::Continue(..) => todo!(),
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
        Statement::Expression(expression) => vec![analyze_poetry(analyzer, *expression).into()],
        Statement::For(..) => todo!(),
        Statement::FunctionAssignment(assignment) => {
            analyze_function_assignment(analyzer, *assignment)
        }
        Statement::Handled(..) => todo!(),
        Statement::If(if_statement) => analyze_if_statement(analyzer, *if_statement),
        Statement::Let(..) => todo!(),
        Statement::Match(..) => todo!(),
        Statement::Resume(..) => todo!(),
        Statement::Return(..) => todo!(),
        Statement::While(..) => todo!(),
        Statement::Yield(..) => todo!(),
    }
}
