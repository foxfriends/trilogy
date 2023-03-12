use super::*;
use crate::{Analyzer, LexicalError};
use trilogy_lexical_ir::{
    Assignment, BinaryDirection, BinaryOperation, Code, Direction, Evaluation, Id, LValue,
    Reference, Step, Value, Violation,
};
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
        Statement::FunctionAssignment(assignment) => {
            let span = assignment.span();
            let function = match analyzer.scope().find(assignment.function.as_ref()) {
                Some(id) => Evaluation {
                    span: assignment.function.span(),
                    value: Value::Dereference(Box::new(Reference {
                        span: assignment.function.span(),
                        target: id,
                    })),
                },
                None => {
                    analyzer.error(LexicalError::UnresolvedIdentifier {
                        span: assignment.function.span(),
                        name: assignment.function.as_ref().to_owned(),
                    });
                    Evaluation {
                        span: assignment.function.span(),
                        value: Value::StaticResolve(assignment.function.as_ref().to_owned()),
                    }
                }
            };
            let rvalue = assignment
                .arguments
                .into_iter()
                .fold(function, |function, argument| Evaluation {
                    span: function.span.union(argument.span()),
                    value: Value::Apply(Box::new(BinaryOperation::new(
                        function,
                        analyze_poetry(analyzer, argument),
                    ))),
                });
            match analyze_lvalue(analyzer, assignment.lhs) {
                LValue::Rebind(reference) => {
                    let rvalue = Evaluation {
                        span,
                        value: Value::Apply(Box::new(BinaryOperation::new(
                            rvalue,
                            Evaluation {
                                span,
                                value: Value::Dereference(Box::new(reference.clone())),
                            },
                        ))),
                    };
                    steps.push(Code::Modification(Box::new(Assignment {
                        span,
                        lvalue: LValue::Rebind(reference),
                        rvalue,
                    })))
                }
                // Fancy assignment to a member expression is a bit harder, as we don't
                // want to double-evaluate either portion.
                LValue::Member {
                    span: lvalue_span,
                    container,
                    property,
                } => {
                    let container_id = Reference {
                        span: container.span,
                        target: Id::new_temporary(container.span),
                    };
                    let property_id = Reference {
                        span: property.span,
                        target: Id::new_temporary(property.span),
                    };
                    // Assign container to temporary
                    steps.push(Code::Direction(Box::new(Direction {
                        span: container.span,
                        body: Step::Unification(Box::new(BinaryOperation {
                            lhs: Evaluation {
                                span: container.span,
                                value: Value::Declaration(Box::new(container_id.clone())),
                            },
                            rhs: container,
                        })),
                    })));
                    // Assign property to temporary
                    steps.push(Code::Direction(Box::new(Direction {
                        span: property.span,
                        body: Step::Unification(Box::new(BinaryOperation::new(
                            Evaluation {
                                span: property.span,
                                value: Value::Declaration(Box::new(property_id.clone())),
                            },
                            property,
                        ))),
                    })));
                    let container_ref = Evaluation {
                        span: container_id.span,
                        value: Value::Dereference(Box::new(container_id)),
                    };
                    let property_ref = Evaluation {
                        span: property_id.span,
                        value: Value::Dereference(Box::new(property_id)),
                    };
                    let access = Evaluation {
                        span: lvalue_span,
                        value: Value::Access(Box::new(BinaryOperation::new(
                            container_ref.clone(),
                            property_ref.clone(),
                        ))),
                    };
                    let rvalue = Evaluation {
                        span,
                        value: Value::Apply(Box::new(BinaryOperation::new(rvalue, access))),
                    };
                    // Assign into pre-evaluated version of lvalue.
                    let lvalue = LValue::Member {
                        span,
                        container: container_ref,
                        property: property_ref,
                    };
                    steps.push(Code::Modification(Box::new(Assignment {
                        span,
                        lvalue,
                        rvalue,
                    })))
                }
            }
        }
        _ => todo!(),
    }
    steps
}
