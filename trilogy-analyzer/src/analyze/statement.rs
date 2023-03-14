use super::*;
use crate::{Analyzer, LexicalError};
use trilogy_lexical_ir::{Assignment, Code, LValue, Reference, Step, Value, Violation};
use trilogy_parser::syntax::{AssignmentStrategy, Statement};
use trilogy_parser::Spanned;

pub(super) fn analyze_statement(analyzer: &mut Analyzer, statement: Statement) -> Vec<Code> {
    let mut steps: Vec<Code> = vec![];

    macro_rules! push {
        ($val:expr) => {
            steps.push($val.into())
        };
    }

    match statement {
        Statement::Assert(assertion) => {
            let whole_span = assertion.span();
            let span = assertion.assertion.span();
            let condition =
                Step::evaluation(analyze_poetry(analyzer, assertion.assertion)).at(span);
            let violation = Step::violation(Violation::AssertionError(
                assertion
                    .message
                    .map(|message| analyze_poetry(analyzer, message))
                    .unwrap(), // TODO: don't unwrap, and put in a default
            ))
            .at(whole_span);
            push!(Step::selection(condition, violation).at(span));
        }
        Statement::Block(block) => push!(analyze_prose(analyzer, *block)),
        Statement::Expression(expression) => {
            push!(analyze_poetry(analyzer, *expression))
        }
        Statement::Assignment(assignment)
            if matches!(assignment.strategy, AssignmentStrategy::Direct(..)) =>
        {
            push!(Assignment::new(
                assignment.span(),
                analyze_lvalue(analyzer, assignment.lhs),
                analyze_poetry(analyzer, assignment.rhs),
            ))
        }
        Statement::End(end_statement) => {
            push!(Step::Contradiction.at(end_statement.span()))
        }
        Statement::Exit(exit_statement) => {
            let span = exit_statement.span();
            push!(Step::violation(Violation::Exit(analyze_poetry(
                analyzer,
                exit_statement.expression,
            )))
            .at(span))
        }
        Statement::FunctionAssignment(assignment) => {
            let span = assignment.span();
            let function = match analyzer.scope().find(assignment.function.as_ref()) {
                Some(id) => Value::dereference(Reference::new(assignment.function.span(), id))
                    .at(assignment.function.span()),
                None => {
                    analyzer.error(LexicalError::UnresolvedIdentifier {
                        span: assignment.function.span(),
                        name: assignment.function.as_ref().to_owned(),
                    });
                    Value::static_resolve(assignment.function.as_ref())
                        .at(assignment.function.span())
                }
            };
            let rvalue = assignment
                .arguments
                .into_iter()
                .fold(function, |function, argument| {
                    let span = function.span.union(argument.span());
                    Value::apply(function, analyze_poetry(analyzer, argument)).at(span)
                });
            match analyze_lvalue(analyzer, assignment.lhs) {
                LValue::Rebind(reference) => {
                    let rvalue =
                        Value::apply(rvalue, Value::dereference(reference.clone()).at(span))
                            .at(span);
                    push!(Assignment::new(span, LValue::Rebind(reference), rvalue));
                }
                // Fancy assignment to a member expression is a bit harder, as we don't
                // want to double-evaluate either portion.
                LValue::Member {
                    span: lvalue_span,
                    container,
                    property,
                } => {
                    let container_id = Reference::temp(container.span);
                    let property_id = Reference::temp(property.span);
                    // Assign container to temporary
                    let container_span = container.span;
                    push!(Step::unification(
                        Value::declaration(container_id.clone()).at(container.span),
                        container,
                    )
                    .at(container_span));
                    // Assign property to temporary
                    let property_span = property.span;
                    push!(Step::unification(
                        Value::declaration(property_id.clone()).at(property.span),
                        property,
                    )
                    .at(property_span));
                    let container_ref = Value::dereference(container_id).at(container_span);
                    let property_ref = Value::dereference(property_id).at(property_span);
                    let access =
                        Value::access(container_ref.clone(), property_ref.clone()).at(lvalue_span);
                    let rvalue = Value::apply(rvalue, access).at(span);
                    // Assign into pre-evaluated version of lvalue.
                    let lvalue = LValue::member(span, container_ref, property_ref);
                    push!(Assignment::new(span, lvalue, rvalue));
                }
            }
        }
        _ => todo!(),
    }
    steps
}
