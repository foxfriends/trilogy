use super::*;
use crate::{Analyzer, LexicalError};
use trilogy_lexical_ir::{Assignment, Code, LValue, Reference, Step, Value};
use trilogy_parser::syntax::FunctionAssignment;
use trilogy_parser::Spanned;

pub(super) fn analyze_function_assignment(
    analyzer: &mut Analyzer,
    assignment: FunctionAssignment,
) -> Vec<Code> {
    let mut steps = vec![];

    let span = assignment.span();
    let function = match analyzer.scope().find(assignment.function.as_ref()) {
        Some(id) => Value::dereference(Reference::new(assignment.function.span(), id))
            .at(assignment.function.span()),
        None => {
            analyzer.error(LexicalError::UnresolvedIdentifier {
                span: assignment.function.span(),
                name: assignment.function.as_ref().to_owned(),
            });
            Value::static_resolve(assignment.function.as_ref()).at(assignment.function.span())
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
                Value::apply(rvalue, Value::dereference(reference.clone()).at(span)).at(span);
            steps.push(Assignment::new(span, LValue::Rebind(reference), rvalue).into());
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
            steps.push(
                Step::unification(
                    Value::declaration(container_id.clone()).at(container.span),
                    container,
                )
                .at(container_span)
                .into(),
            );
            // Assign property to temporary
            let property_span = property.span;
            steps.push(
                Step::unification(
                    Value::declaration(property_id.clone()).at(property.span),
                    property,
                )
                .at(property_span)
                .into(),
            );
            let container_ref = Value::dereference(container_id).at(container_span);
            let property_ref = Value::dereference(property_id).at(property_span);
            let access = Value::access(container_ref.clone(), property_ref.clone()).at(lvalue_span);
            let rvalue = Value::apply(rvalue, access).at(span);
            // Assign into pre-evaluated version of lvalue.
            let lvalue = LValue::member(span, container_ref, property_ref);
            steps.push(Assignment::new(span, lvalue, rvalue).into());
        }
    }

    steps
}
