use super::*;
use crate::{Converter, Error};
use source_span::Span;
use trilogy_parser::{Spanned, syntax};

#[derive(Clone, Debug)]
pub struct Assignment {
    pub lhs: Expression,
    pub rhs: Expression,
}

impl Assignment {
    pub(super) fn convert(
        converter: &mut Converter,
        ast: syntax::AssignmentStatement,
    ) -> Expression {
        use syntax::AssignmentStrategy::*;
        let span = ast.span();
        let lhs = Expression::convert(converter, ast.lhs);
        let rhs = Expression::convert(converter, ast.rhs);

        let op = match ast.strategy {
            Direct(..) => {
                if let expression::Value::Reference(id) = &lhs.value
                    && !id.is_mutable
                {
                    converter.error(Error::AssignedImmutableBinding {
                        name: *id.clone(),
                        assignment: span,
                    });
                }
                return Expression::assignment(span, Assignment { lhs, rhs });
            }
            And(token) => Expression::builtin(token.span, Builtin::And),
            Or(token) => Expression::builtin(token.span, Builtin::Or),
            Add(token) => Expression::builtin(token.span, Builtin::Add),
            Subtract(token) => Expression::builtin(token.span, Builtin::Subtract),
            Multiply(token) => Expression::builtin(token.span, Builtin::Multiply),
            Divide(token) => Expression::builtin(token.span, Builtin::Divide),
            Remainder(token) => Expression::builtin(token.span, Builtin::Remainder),
            Power(token) => Expression::builtin(token.span, Builtin::Power),
            IntDivide(token) => Expression::builtin(token.span, Builtin::IntDivide),
            BitwiseAnd(token) => Expression::builtin(token.span, Builtin::BitwiseAnd),
            BitwiseOr(token) => Expression::builtin(token.span, Builtin::BitwiseOr),
            BitwiseXor(token) => Expression::builtin(token.span, Builtin::BitwiseXor),
            LeftShift(token) => Expression::builtin(token.span, Builtin::LeftShift),
            RightShift(token) => Expression::builtin(token.span, Builtin::RightShift),
            LeftShiftExtend(token) => Expression::builtin(token.span, Builtin::LeftShiftExtend),
            RightShiftExtend(token) => Expression::builtin(token.span, Builtin::RightShiftExtend),
            LeftShiftContract(token) => Expression::builtin(token.span, Builtin::LeftShiftContract),
            RightShiftContract(token) => {
                Expression::builtin(token.span, Builtin::RightShiftContract)
            }
            Glue(token) => Expression::builtin(token.span, Builtin::Glue),
            Compose(token) => Expression::builtin(token.span, Builtin::Compose),
            RCompose(token) => Expression::builtin(token.span, Builtin::RCompose),
            Access(token) => Expression::builtin(token.span, Builtin::Access),
            Cons(token) => Expression::builtin(token.span, Builtin::Cons),
        };

        match lhs.deconstruct_lvalue() {
            Ok((receiver, access_span, property)) => {
                let receiver_span = receiver.span;
                let receiver_id = Identifier::temporary(converter, receiver_span);
                let receiver_expression = Expression::reference(receiver_span, receiver_id.clone());
                let receiver_pattern = Expression::reference(receiver_span, receiver_id);
                let receiver_query =
                    Query::direct(receiver_span, Unification::new(receiver_pattern, receiver));

                let property_span = property.span;
                let property_id = Identifier::temporary(converter, property_span);
                let property_expression = Expression::reference(property_span, property_id.clone());
                let property_pattern = Expression::reference(property_span, property_id);
                let property_query =
                    Query::direct(property_span, Unification::new(property_pattern, property));

                let lhs = Expression::builtin(access_span, Builtin::Access)
                    .apply_to(access_span.union(receiver_span), receiver_expression)
                    .apply_to(property_span.union(receiver_span), property_expression);

                let op_span = op.span;
                let rhs = op
                    .apply_to(op_span.union(lhs.span), lhs.clone())
                    .apply_to(span, rhs);
                Expression::assignment(span, Self { lhs, rhs })
                    .in_let(span, property_query)
                    .in_let(span, receiver_query)
            }
            Err(lhs) => {
                let id = lhs.unwrap_reference();
                if !id.is_mutable {
                    converter.error(Error::AssignedImmutableBinding {
                        name: id.clone(),
                        assignment: span,
                    });
                }
                let op_span = op.span;
                let rhs = op
                    .apply_to(op_span.union(lhs.span), lhs.clone())
                    .apply_to(span, rhs);
                Expression::assignment(span, Self { lhs, rhs })
            }
        }
    }

    pub(super) fn convert_function(
        converter: &mut Converter,
        ast: syntax::FunctionAssignment,
    ) -> Expression {
        let span = ast.span();
        let lhs = Expression::convert(converter, ast.lhs);
        let function = Identifier::declared(converter, &ast.function).unwrap_or_else(|| {
            converter.error(Error::UnboundIdentifier {
                name: ast.function.clone(),
            });
            // TODO: All these missed declarations probably shouldn't just declare it?
            // Maybe some smarter recovery is possible.
            Identifier::declare(converter, ast.function.clone())
        });
        let function = Expression::reference(function.span, function);
        let function = ast
            .arguments
            .into_iter()
            .map(|arg| Expression::convert(converter, arg))
            .fold(function, |func, arg| {
                let span = func.span.union(arg.span);
                func.apply_to(span, arg)
            });

        match lhs.deconstruct_lvalue() {
            Ok((receiver, access_span, property)) => {
                let receiver_span = receiver.span;
                let receiver_id = Identifier::temporary(converter, receiver_span);
                let receiver_expression = Expression::reference(receiver_span, receiver_id.clone());
                let receiver_pattern = Expression::reference(receiver_span, receiver_id);
                let receiver_query =
                    Query::direct(receiver_span, Unification::new(receiver_pattern, receiver));

                let property_span = property.span;
                let property_id = Identifier::temporary(converter, property_span);
                let property_expression = Expression::reference(property_span, property_id.clone());
                let property_pattern = Expression::reference(property_span, property_id);
                let property_query =
                    Query::direct(property_span, Unification::new(property_pattern, property));

                let lhs = Expression::builtin(access_span, Builtin::Access)
                    .apply_to(access_span.union(receiver_span), receiver_expression)
                    .apply_to(property_span.union(receiver_span), property_expression);
                let rhs = function.apply_to(span, lhs.clone());
                Expression::assignment(span, Self { lhs, rhs })
                    .in_let(span, property_query)
                    .in_let(span, receiver_query)
            }
            Err(lhs) => {
                let id = lhs.unwrap_reference();
                if !id.is_mutable {
                    converter.error(Error::AssignedImmutableBinding {
                        name: id.clone(),
                        assignment: span,
                    });
                }
                let rhs = function.apply_to(span, lhs.clone());
                Expression::assignment(span, Self { lhs, rhs })
            }
        }
    }
}

impl Expression {
    fn deconstruct_lvalue(self) -> Result<(Self, Span, Self), Self> {
        match self.value {
            expression::Value::Reference(..) => Err(self),
            expression::Value::Application(app) => {
                let property = app.argument;

                let expression::Value::Application(app) = app.function.value else {
                    panic!("lvalue is not valid: not a double application");
                };

                let receiver = app.argument;
                let access_span = match app.function.value {
                    expression::Value::Builtin(Builtin::Access) => app.function.span,
                    _ => panic!("lvalue is not valid: not a member access"),
                };

                Ok((receiver, access_span, property))
            }
            _ => panic!("lvalue is not valid: not an application or reference"),
        }
    }

    fn unwrap_reference(&self) -> &Identifier {
        match &self.value {
            expression::Value::Reference(id) => id,
            _ => unreachable!(),
        }
    }
}
