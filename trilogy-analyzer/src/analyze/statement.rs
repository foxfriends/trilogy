use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Assignment, Code, Cond, Step, Value, Violation};
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
        Statement::Break(token) => vec![Value::r#break(Value::Unit.at(token.span()))
            .at(token.span())
            .into()],
        Statement::Cancel(cancel) => {
            let span = cancel.span();
            let evaluation = match cancel.expression {
                Some(expression) => analyze_poetry(analyzer, expression),
                None => Value::Unit.at(span),
            };
            vec![Value::cancel(evaluation).at(span).into()]
        }
        Statement::Continue(token) => vec![Value::r#continue(Value::Unit.at(token.span()))
            .at(token.span())
            .into()],
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
        Statement::Match(match_statement) => analyze_match_statement(analyzer, *match_statement),
        Statement::Resume(resume) => {
            let span = resume.span();
            let evaluation = match resume.expression {
                Some(expression) => analyze_poetry(analyzer, expression),
                None => Value::Unit.at(span),
            };
            vec![Value::resume(evaluation).at(span).into()]
        }
        Statement::Return(return_statement) => {
            let span = return_statement.span();
            let evaluation = match return_statement.expression {
                Some(expression) => analyze_poetry(analyzer, expression),
                None => Value::Unit.at(span),
            };
            vec![Value::r#return(evaluation).at(span).into()]
        }
        Statement::While(while_statement) => {
            let span = while_statement.span();
            let condition = analyze_poetry(analyzer, while_statement.condition);
            let body = analyze_prose(analyzer, while_statement.body);
            let cond = Cond::new_loop(vec![condition.into()], vec![body]);
            vec![Value::cond(vec![cond]).at(span).into()]
        }
        Statement::Yield(yield_statement) => {
            let span = yield_statement.span();
            let evaluation = analyze_poetry(analyzer, yield_statement.expression);
            vec![Value::r#yield(evaluation).at(span).into()]
        }
    }
}
