use super::*;
use crate::Analyzer;
use trilogy_lexical_ir::{Assignment, BinaryDirection, Code, Direction, Step, Violation};
use trilogy_parser::syntax::Statement;
use trilogy_parser::Spanned;

pub(super) fn analyze_statement(analyzer: &mut Analyzer, statement: Statement) -> Vec<Code> {
    let mut steps = vec![];
    match statement {
        Statement::Assert(assertion) => {
            let span = assertion.span();
            let condition = Direction {
                span: assertion.assertion.span(),
                body: Step::Evaluation(Box::new(analyze_poetry(analyzer, assertion.assertion))),
            };

            let violation = Direction {
                span,
                body: Step::Violation(Violation::AssertionError(Box::new(
                    assertion
                        .message
                        .map(|message| analyze_poetry(analyzer, message))
                        .unwrap(), // TODO: don't unwrap, and put in a default
                ))),
            };
            steps.push(Code::Direction(Box::new(Direction {
                span,
                body: Step::Selection(Box::new(BinaryDirection::new(condition, violation))),
            })));
        }
        Statement::Block(block) => steps.push(analyze_prose(analyzer, *block)),
        Statement::Expression(expression) => steps.push(Code::Evaluation(Box::new(
            analyze_poetry(analyzer, *expression),
        ))),
        Statement::Assignment(assignment) => steps.push(Code::Modification(Box::new(Assignment {
            span: assignment.span(),
            lvalue: analyze_lvalue(analyzer, assignment.lhs),
            rvalue: analyze_poetry(analyzer, assignment.rhs),
        }))),
        Statement::End(end_statement) => steps.push(Code::Direction(Box::new(Direction {
            span: end_statement.span(),
            body: Step::Contradiction,
        }))),
        Statement::Exit(exit_statement) => steps.push(Code::Direction(Box::new(Direction {
            span: exit_statement.span(),
            body: Step::Violation(Violation::Exit(Box::new(analyze_poetry(
                analyzer,
                exit_statement.expression,
            )))),
        }))),
        _ => todo!(),
    }
    steps
}
